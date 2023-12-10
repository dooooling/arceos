extern crate alloc;

use alloc::string::String;
use alloc::{vec, vec::Vec};
use core::alloc::Layout;
use core::fmt::{Debug, Display, Formatter};
use core::mem::size_of;
use core::ptr::NonNull;
use core::time::Duration;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

use axalloc::global_allocator;

// 10 milliseconds
const XECP_LEGACY_TIMEOUT: usize = 10;
const PROTO_USB2: u8 = 0;
const PROTO_USB3: u8 = 1;

const PROTO_HSO: u8 = 1 << 1;
const PROTO_HAS_PAIR: u8 = 1 << 2;
const PROTO_ACTIVE: u8 = 1 << 3;

#[derive(Debug)]
pub struct XHCIRegister {
    base: NonNull<u8>,
    capability: NonNull<CapabilityRegisters>,
    operational: NonNull<OperationalRegisters>,
    xecps: Vec<NonNull<Xecp>>,
    max_ports: u32,
}

#[derive(Default, Clone, Copy, Debug)]
struct Port {
    // port_info flags below
    flags: u8,
    // zero based offset to other speed port
    other_port_num: u8,
    // offset of this port within this protocol
    offset: u8,
    reserved: u8,
}

/// qemu 访问寄存器需要32位（dword）对齐  https://forum.osdev.org/viewtopic.php?p=309454
impl XHCIRegister {
    pub fn new(addr: *mut u8) -> Self {
        let base: NonNull<u8> = NonNull::new(addr).unwrap().cast();
        let capability: NonNull<CapabilityRegisters> = NonNull::new(addr).unwrap().cast();
        let operational: NonNull<OperationalRegisters> = unsafe {
            NonNull::new(
                base.as_ptr().offset(
                    capability
                        .as_ref()
                        .capl_version
                        .read(CAPLVERSION::caplength) as isize,
                ),
            )
            .unwrap()
            .cast()
        };
        error!("cap len : {}", unsafe {
            capability
                .as_ref()
                .capl_version
                .read(CAPLVERSION::caplength)
        });
        error!("hci version : {}", unsafe {
            capability
                .as_ref()
                .capl_version
                .read(CAPLVERSION::hciversion)
        });
        let mut xecps = vec![];
        unsafe {
            let mut xecp_offset = capability.as_ref().hccparams1.read(HCCPARAMS1::XECP);
            loop {
                let xecp: NonNull<Xecp> =
                    NonNull::new((addr as *mut u32).offset(xecp_offset as isize))
                        .unwrap()
                        .cast();
                xecps.push(xecp);
                let next_offset = xecp.as_ref().DW0.read(XEPCNORMAL::NEXTOFFSET);
                info!("next offset : {:#X}", next_offset);
                if next_offset == 0 {
                    break;
                }
                xecp_offset += next_offset;
            }
        }
        let max_ports = unsafe { capability.as_ref().hcsparams1.read(HCSPARAMS1::MaxPorts) };
        Self {
            base,
            capability,
            operational,
            xecps,
            max_ports,
        }
    }

    fn capability(&self) -> &CapabilityRegisters {
        unsafe { self.capability.as_ref() }
    }
    fn operational(&self) -> &OperationalRegisters {
        unsafe { self.operational.as_ref() }
    }
    fn xecps(&self) -> Vec<&Xecp> {
        unsafe { self.xecps.iter().map(|xecp| xecp.as_ref()).collect() }
    }

    /// 初始化 xHCI 控制器
    pub fn init(&self, gsi: u32) {
        // 控制器复位
        self.xhci_rest();

        //释放bios对控制器的控制权
        self.stop_legacy();

        let cap = self.capability();
        let max_ports = cap.hcsparams1.read(HCSPARAMS1::MaxPorts);
        info!("xHCI detected ports : {}", max_ports);
        let mut ports = self.pair_ports();
        let op = self.operational();
        let page_size = (op.PAGESIZE.get() & 0xFFFF) << 12;
        let max_slots = cap.hcsparams1.read(HCSPARAMS1::MaxSlots);
        info!("xHCI page size : {:#X}", page_size);
        let addr = unsafe { global_allocator().alloc(Layout::from_size_align_unchecked(2048, 64)) }
            .unwrap()
            .as_ptr();
        for i in 0..2048 {
            unsafe { addr.offset(i).write(0) }
        }
        info!("mem addr : {:#X}", addr as u64);
        op.DCBAAP.set(addr as u64);

        let max_scratch_buffs = self.capability().hcsparams2.read(HCSPARAMS2::MSB_L5);
        info!("max scratch buffs : {:#X}", max_scratch_buffs);

        if max_scratch_buffs > 0 {
            let scratch_buff_array_start = unsafe {
                global_allocator().alloc(Layout::from_size_align_unchecked(
                    (max_scratch_buffs * 8) as usize,
                    64,
                ))
            }
            .unwrap()
            .as_ptr();
            let scratch_buff_start = unsafe {
                global_allocator().alloc(Layout::from_size_align_unchecked(
                    (max_scratch_buffs * page_size) as usize,
                    page_size as usize,
                ))
            }
            .unwrap()
            .as_ptr();
            unsafe {
                addr.cast::<u64>().write(scratch_buff_array_start as u64);
                for i in 0..max_scratch_buffs {
                    scratch_buff_array_start
                        .cast::<u64>()
                        .offset(i as isize)
                        .write((scratch_buff_start as usize + (i * page_size) as usize) as u64);
                }
            }
        }

        let cmnd_ring_addr = create_ring(128);
        let cmnd_trb_addr = cmnd_ring_addr;
        let cmnd_trb_cycle = 1;

        // Command Ring Control Register
        op.CRCR.set(cmnd_ring_addr as u64 | 1);
        // Configure Register
        op.CONFIG.set(max_slots);
        // Device Notification Control (only bit 1 is allowed)
        op.DNCTRL.set(1 << 1);

        // Initialize the interrupters
        let max_event_segs = 1 << cap.hcsparams2.read(HCSPARAMS2::ERSTMAX);
        let max_interrupters = cap.hcsparams1.read(HCSPARAMS1::MaxIntrs);

        let (cur_event_ring_addr, event_ring_addr) = create_event_ring(4096);
        let cur_event_ring_cycle = 1;

        // rutime register
        let rtsoff = cap.rtsoff.read(RTSOFF::OFFSET);
        unsafe {
            let ir_set = self
                .base
                .as_ptr()
                .offset((rtsoff + 0x20) as isize)
                .cast::<InterrupterRegisterSet>();
            (*ir_set).IMAN.write(IMAN::IE::SET);
            (*ir_set).IMAN.write(IMAN::IP::SET);
            (*ir_set).IMOD.write(IMOD::IMODC::CLEAR);
            (*ir_set).IMOD.write(IMOD::IMODI::CLEAR);
            (*ir_set).ERSTSZ.set(1);
            (*ir_set)
                .ERSTBA
                .write(ERSTBA::BASE_ADDR.val(cur_event_ring_addr as u64));
            (*ir_set)
                .ERDP
                .write(ERDP::POINTER.val(event_ring_addr as u64));
        }
        op.USBSTS.set((1 << 10) | (1 << 4) | (1 << 3) | (1 << 2));
        axhal::irq::register_handler((gsi + 0x20) as usize, test_handler);
        op.USBCMD.set((1 << 3) | (1 << 2) | (1 << 0));
        axhal::time::busy_wait(Duration::from_millis(100));

        // loop through the ports, starting with the USB3 ports
        // for (i = 0; i < ndp; i++) {
        //     if (xHCI_IS_USB3_PORT(i) && xHCI_IS_ACTIVE(i)) {
        //         // power and reset the port
        //         if (xhci_reset_port(i))
        //         // if the reset was good, get the descriptor
        //         // if the reset was bad, the reset routine will mark this port as inactive,
        //         //  and mark the USB2 port as active.
        //         xhci_get_descriptor(i);
        //     }
        // }
        ports
            .iter_mut()
            .filter(|p| {
                (p.flags & PROTO_USB3) == PROTO_USB3 && (p.flags & PROTO_ACTIVE) == PROTO_ACTIVE
            })
            .for_each(|p| {
                if self.xhci_reset_port(p) {
                    error!(" rest port success!");
                    self.xhci_get_descriptor(p);
                } else {
                    error!(" rest port error!");
                }
            });
        for i in 0..ports.len() {}
    }
    fn xhci_rest(&self) {
        let opt_register = unsafe { self.operational.as_ref() };
        //reset the controller

        let delay = Duration::from_millis(1);
        opt_register.USBCMD.set(1 << 1);

        let mut times = 500;
        while (opt_register.USBCMD.get() & (1 << 1)) == 1 {
            axhal::time::busy_wait(delay);
            times -= 1;
            if times == 0 {
                panic!("xHCI controller rest bit failed to clear!")
            }
        }
        info!("xHCI controller rest bit clear success!")
    }
    fn stop_legacy(&self) {
        let option_legacy = self.xecps.iter().find_map(|xecp| unsafe {
            if xecp.as_ref().DW0.read(XEPCNORMAL::ID) == 1 {
                Some(xecp.cast::<XecpLegacy>())
            } else {
                None
            }
        });

        let mut times = XECP_LEGACY_TIMEOUT;
        let delay = Duration::from_millis(1);

        if let Some(legacy) = option_legacy {
            unsafe {
                legacy.as_ref().DW0.write(XECPLEGACY_DW0::SYSTEM_OWNED::SET);
                while legacy.as_ref().DW0.read(XECPLEGACY_DW0::BISO_OWNED) == 1 {
                    axhal::time::busy_wait(delay);
                    times -= 1;
                    if times == 0 {
                        panic!("BIOS Ownership failed to disable!")
                    }
                }
            }
            info!("xHCI BIOS Ownership released!")
        } else {
            info!("xHCI no BIOS Ownership detected!")
        }
    }

    fn pair_ports(&self) -> Vec<Port> {
        let mut ports: Vec<Port> = (0..self.max_ports).map(|_| Port::default()).collect();
        let usb_xecps: Vec<&XecpUSBProtocol> = self
            .xecps
            .iter()
            .filter_map(|xecp| unsafe {
                if xecp.as_ref().DW0.read(XEPCNORMAL::ID) == 2 {
                    Some(xecp.cast::<XecpUSBProtocol>().as_ref())
                } else {
                    None
                }
            })
            .collect();
        let mut ports_usb2 = 0;
        let mut ports_usb3 = 0;
        for x in usb_xecps {
            let offset = x.DW2.read(USBPRTL_DW2::COMP_OFFSET) - 1;
            let count = x.DW2.read(USBPRTL_DW2::COMP_COUNT);
            let define = x.DW2.read(USBPRTL_DW2::PROT_DEF);
            info!(
                "usb protocol offset : {} , count : {} , define : {}",
                offset, count, define
            );

            //usb 2
            if x.DW0.read(USBPRTL_DW0::MAJOR) == 2 {
                for i in 0..count {
                    let mut port = Port {
                        flags: PROTO_USB2,
                        other_port_num: 0,
                        offset: ports_usb2,
                        reserved: 0,
                    };

                    if define & 2 > 0 {
                        port.offset |= PROTO_HSO;
                    }
                    ports_usb2 += 1;
                    ports[(offset + i) as usize] = port;
                }
            } else if x.DW0.read(USBPRTL_DW0::MAJOR) == 3 {
                //usb 3
                for i in 0..count {
                    let port = Port {
                        flags: PROTO_USB3,
                        other_port_num: 0,
                        offset: ports_usb3,
                        reserved: 0,
                    };
                    ports_usb3 += 1;
                    ports[(offset + i) as usize] = port;
                    info!("insert usb3 offset {}", offset + i);
                }
            }
        }
        for i in 0..ports.len() {
            for j in 0..ports.len() {
                if ports[i].offset == ports[j].offset && (ports[i].flags & 1 != ports[j].flags & 1)
                {
                    ports[i].other_port_num = j as u8;
                    ports[i].flags |= PROTO_HAS_PAIR;
                    ports[j].other_port_num = i as u8;
                    ports[j].flags |= PROTO_HAS_PAIR;
                }
            }
        }

        for port in ports.iter_mut() {
            if port.flags & 1 == 1
                || (port.flags & 1 == 0 && port.flags & PROTO_HAS_PAIR == PROTO_HAS_PAIR)
            {
                port.flags |= PROTO_ACTIVE;
            }
        }
        ports
    }

    fn xhci_reset_port(&self, port: &mut Port) -> bool {
        let mut ret = false;

        // power the port?
        let port_set = unsafe {
            self.operational
                .cast::<u8>()
                .as_ptr()
                .offset(0x400)
                .cast::<PortRegisterSet>()
                .offset(port.offset as isize)
        };
        info!("port address {:#X}", port_set as usize);
        let portsc = unsafe { &(*port_set).PORTSC };
        if portsc.get() & (1 << 9) == 0 {
            portsc.set(1 << 9);
            axhal::time::busy_wait(Duration::from_millis(20));
            if portsc.get() & (1 << 9) == 0 {
                return false;
            }
        }

        // we need to make sure that the status change bits are clear
        portsc.set((1 << 9) | ((1 << 17) | (1 << 18) | (1 << 20) | (1 << 21) | (1 << 22)));

        // set bit 4 (USB2) or 31 (USB3) to reset the port
        if (port.flags & PROTO_USB3) == PROTO_USB3 {
            portsc.set((1 << 9) | (1 << 31));
        } else {
            portsc.set((1 << 9) | (1 << 4));
        }

        // wait for bit 21 to set
        let mut timeout = 500;
        while timeout > 0 {
            if (portsc.get() & (1 << 21)) > 0 {
                break;
            }
            timeout -= 1;
            axhal::time::busy_wait(Duration::from_millis(1));
        }

        // if we didn't time out
        if timeout > 0 {
            // reset recovery time
            axhal::time::busy_wait(Duration::from_millis(3));

            // if after the reset, the enable bit is non zero, there was a successful reset/enable
            if (portsc.get() & (1 << 1)) > 0 {
                // clear the status change bit(s)
                portsc.set((1 << 9) | ((1 << 17) | (1 << 18) | (1 << 20) | (1 << 21) | (1 << 22)));
                // success
                ret = true;
            }
        }

        ret
    }

    fn xhci_get_descriptor(&self, port: &mut Port) {
        let port_set = unsafe {
            self.operational
                .cast::<u8>()
                .as_ptr()
                .offset(0x400)
                .cast::<PortRegisterSet>()
                .offset(port.offset as isize)
        };
        info!("port address {:#X}", port_set as usize);
        let portsc = unsafe { &(*port_set).PORTSC };
        let speed = (portsc.get() & (0xF << 10)) >> 10;

        // send the command and wait for it to return
        // let trb = Trb::(5).unwrap();
        // Trb{
        //     param: ReadWrite::try_from(1).unwrap(),
        //     status: ReadWrite::try_from(1).unwrap(),
        //     command: ReadWrite::try_from(1).unwrap(),
        // };
        // struct xHCI_TRB trb;
        // trb.param = 0;
        // trb.status = 0;
        // trb.command = TRB_SET_STYPE(0) | TRB_SET_TYPE(ENABLE_SLOT);
        // if (xhci_send_command(&trb, TRUE))
        // return FALSE;
    }
}

fn test_handler() {
    error!("usb irq handler");
}

const TRB_LINK_CMND: u32 = ((6 & 0x3F) << 10 | 0 << 5 | 0 << 4 | 0 << 1 | 1 << 0);

fn create_ring(trbs: usize) -> *mut u8 {
    unsafe {
        let addr = global_allocator()
            .alloc(Layout::from_size_align_unchecked(
                (trbs * size_of::<Trb>()),
                64,
            ))
            .unwrap()
            .as_ptr();
        let link_trb = addr
            .offset(((trbs - 1) * size_of::<Trb>()) as isize)
            .cast::<Trb>();
        (*link_trb).param.set(addr as u64);
        (*link_trb).status.set((0 << 22) | 0);
        (*link_trb).command.set(TRB_LINK_CMND);
        addr
    }
}

fn create_event_ring(trbs: usize) -> (*mut u8, *mut u8) {
    unsafe {
        let tab_addr = global_allocator()
            .alloc(Layout::from_size_align_unchecked(64, 64))
            .unwrap()
            .as_ptr();
        let addr = global_allocator()
            .alloc(Layout::from_size_align_unchecked(
                (trbs * size_of::<Trb>()),
                64,
            ))
            .unwrap()
            .as_ptr();
        tab_addr.cast::<u64>().write(addr as u64);
        tab_addr.cast::<u32>().offset(2).write(trbs as u32);
        tab_addr.cast::<u32>().offset(3).write(0);
        (tab_addr, addr)
    }
}

impl Display for XHCIRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        unsafe {
            let xecps = self.xecps();
            write!(
                f,
                "addr: {:#X} , ver: {:#X} , xecps: {:?}",
                self.base.as_ptr() as usize,
                self.capability
                    .as_ref()
                    .capl_version
                    .read(CAPLVERSION::hciversion),
                xecps
            )
        }
    }
}
register_structs! {
     Trb{
        (0x00 => param:ReadWrite<u64>),
        (0x08 => status:ReadWrite<u32>),
        (0x0C => command:ReadWrite<u32>),
        (0x10 => @END),
    }
}
register_structs! {
     CapabilityRegisters{
        (0x00 => capl_version:ReadOnly<u32,CAPLVERSION::Register>),
        (0x04 => hcsparams1:ReadOnly<u32, HCSPARAMS1::Register>),
        (0x08 => hcsparams2:ReadOnly<u32, HCSPARAMS2::Register>),
        (0x0C => hcsparams3:ReadOnly<u32, HCSPARAMS3::Register>),
        (0x10 => hccparams1:ReadOnly<u32,HCCPARAMS1::Register>),
        (0x14 => dboff:ReadOnly<u32, DBOFF::Register>),
        (0x18 => rtsoff:ReadOnly<u32, RTSOFF::Register>),
        (0x1C => hccparams2:ReadOnly<u32, HCCPARAMS2::Register>),
        (0x20 => @END),
    }
}
register_structs! {
     InterrupterRegisterSet{
        (0x00 => IMAN:ReadWrite<u32,IMAN::Register>),
        (0x04 => IMOD:ReadWrite<u32, IMOD::Register>),
        (0x08 => ERSTSZ:ReadWrite<u32, ERSTSZ::Register>),
        (0x0C => _RsvdP:ReadWrite<u32>),
        (0x10 => ERSTBA:ReadWrite<u64,ERSTBA::Register>),
        (0x18 => ERDP:ReadWrite<u64, ERDP::Register>),
        (0x20 => @END),
    }
}
register_structs! {
     PortRegisterSet{
        (0x00 => PORTSC:ReadWrite<u32>),
        (0x04 => PORTPMSC:ReadWrite<u32>),
        (0x08 => PORTLI:ReadWrite<u32>),
        (0x0C => PORTHLPMC:ReadWrite<u32>),
        (0x10 => @END),
    }
}

register_bitfields! {
    u32,
    CAPLVERSION[
        caplength OFFSET(0) NUMBITS(8) [],
        hciversion OFFSET(16) NUMBITS(16) [],
    ],
    HCSPARAMS1[
        MaxSlots OFFSET(0) NUMBITS(8) [],
        MaxIntrs OFFSET(8) NUMBITS(11) [],
        MaxPorts OFFSET(24) NUMBITS(8) [],
    ],
    HCSPARAMS2[
        IST OFFSET(0) NUMBITS(4) [],
        ERSTMAX OFFSET(4) NUMBITS(4) [],
        MSB_H5 OFFSET(21) NUMBITS(5) [],
        SPR OFFSET(26) NUMBITS(1) [],
        MSB_L5 OFFSET(27) NUMBITS(5) [],
    ],
    HCSPARAMS3[
        U1DEL OFFSET(0) NUMBITS(8) [],
        U2DEL OFFSET(16) NUMBITS(16) [],
    ],
    HCCPARAMS1[
        AC64 OFFSET(0) NUMBITS(1) [],
        BNC OFFSET(1) NUMBITS(1) [],
        CSZ OFFSET(2) NUMBITS(1) [],
        PPC OFFSET(3) NUMBITS(1) [],
        PIND OFFSET(4) NUMBITS(1) [],
        LHRC OFFSET(5) NUMBITS(1) [],
        LTC OFFSET(6) NUMBITS(1) [],
        NSS OFFSET(7) NUMBITS(1) [],
        PAE OFFSET(8) NUMBITS(1) [],
        SPC OFFSET(9) NUMBITS(1) [],
        SEC OFFSET(10) NUMBITS(1) [],
        CFC OFFSET(11) NUMBITS(1) [],
        MaxPSASize OFFSET(12) NUMBITS(4) [],
        XECP OFFSET(16) NUMBITS(16) [],
    ],
    DBOFF[
       OFFSET OFFSET(2) NUMBITS(30) [],
    ],
    RTSOFF[
       OFFSET OFFSET(5) NUMBITS(27) [],
    ],
    HCCPARAMS2[
        U3C OFFSET(0) NUMBITS(1) [],
        CMC OFFSET(1) NUMBITS(1) [],
        FSC OFFSET(2) NUMBITS(1) [],
        CTC OFFSET(3) NUMBITS(1) [],
        LEC OFFSET(4) NUMBITS(1) [],
        CIC OFFSET(5) NUMBITS(1) [],
    ],
    XECPLEGACY_DW0[
        ID OFFSET(0) NUMBITS(8) [],
        NEXTOFFSET OFFSET(8) NUMBITS(8) [],
        BISO_OWNED OFFSET(16) NUMBITS(1) [],
        SYSTEM_OWNED OFFSET(24) NUMBITS(1) [],
    ],
    XEPCNORMAL[
        ID OFFSET(0) NUMBITS(8) [],
        NEXTOFFSET OFFSET(8) NUMBITS(8) [],
    ],
    USBPRTL_DW0[
        ID OFFSET(0) NUMBITS(8) [],
        NEXTOFFSET OFFSET(8) NUMBITS(8) [],
        MINOR OFFSET(16) NUMBITS(8) [],
        MAJOR OFFSET(24) NUMBITS(8) [],
    ],
    USBPRTL_DW2[
        COMP_OFFSET OFFSET(0) NUMBITS(8) [],
        COMP_COUNT OFFSET(8) NUMBITS(8) [],
        PROT_DEF OFFSET(16) NUMBITS(12) [],
        SPEED_ID_COUNT OFFSET(28) NUMBITS(8) [],
    ],
    IMAN[
        IP OFFSET(0) NUMBITS(1) [],
        IE OFFSET(1) NUMBITS(1) [],
    ],
    IMOD[
        IMODI OFFSET(0) NUMBITS(16) [],
        IMODC OFFSET(16) NUMBITS(16) [],
    ],
    ERSTSZ[
        SIZE OFFSET(0) NUMBITS(16) [],
    ],
}

register_bitfields! {
    u64,
    ERSTBA[
        BASE_ADDR OFFSET(6) NUMBITS(58) [],
    ],
    ERDP[
        DESI OFFSET(0) NUMBITS(2) [],
        EHBI OFFSET(3) NUMBITS(1) [],
        POINTER OFFSET(4) NUMBITS(60) [],
    ],
}
register_structs! {
    OperationalRegisters{
        (0x00 => USBCMD:ReadWrite<u32>),
        (0x04 => USBSTS:ReadWrite<u32>),
        (0x08 => PAGESIZE:ReadOnly<u32>),
        (0x0C => _reserved0),
        (0x14 => DNCTRL:ReadWrite<u32>),
        (0x18 => CRCR:ReadWrite<u64>),
        (0x20 => _reserved1),
        (0x30 => DCBAAP:ReadWrite<u64>),
        (0x38 => CONFIG:ReadWrite<u32>),
        (0x3C => @END),
    }
}

register_structs! {
    Xecp{
        (0x00 => DW0:ReadWrite<u32,XEPCNORMAL::Register>),
        (0x04 => @END),
    }
}
impl Debug for Xecp {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Xecp {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "protocol: {} - {}",
            self.DW0.read(XEPCNORMAL::ID),
            self.DW0.read(XEPCNORMAL::NEXTOFFSET),
        )
    }
}

register_structs! {
    XecpLegacy{
        (0x00 => DW0:ReadWrite<u32, XECPLEGACY_DW0::Register>),
        (0x04 => DW1:ReadWrite<u32>),
        (0x08 => @END),
    }
}

register_structs! {
    XecpUSBProtocol{
        (0x00 => DW0:ReadWrite<u32, USBPRTL_DW0::Register>),

        (0x04 => name:ReadOnly<u32>),

        (0x08 => DW2:ReadOnly<u32, USBPRTL_DW2::Register>),

        (0x0C => portocol_slot_type:ReadOnly<u32>),
        (0x10 => @END),
    }
}

impl Debug for XecpUSBProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for XecpUSBProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "usb protocol: {}{}.{}",
            String::from_utf8_lossy(&self.name.get().to_le_bytes()),
            self.DW0.read(USBPRTL_DW0::MAJOR),
            self.DW0.read(USBPRTL_DW0::MINOR),
        )
    }
}
