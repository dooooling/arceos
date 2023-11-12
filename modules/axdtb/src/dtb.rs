use crate::dtb::DtbParseError::InvalidAddr;
use crate::dtb::GenericNodePrefix::{MEMORY, ROOT, SOC, VIRTIO_MMIO};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Display, Formatter};
use hermit_dtb::Dtb;

pub struct DtbWrapper<'a> {
    dtb: Dtb<'a>,
}
#[derive(Debug)]
pub enum DtbParseError {
    InvalidAddr(usize),
    ParseError,
}

pub enum GenericNodePrefix {
    ROOT,
    MEMORY,
    SOC,
    VIRTIO_MMIO,
}

impl<'a> Into<&'a str> for GenericNodePrefix {
    fn into(self) -> &'a str {
        match self {
            ROOT => "/",
            MEMORY => "memory",
            SOC => "soc",
            VIRTIO_MMIO => "virtio_mmio",
        }
    }
}

pub enum GenericProperty {
    REG,
}

impl<'a> Into<&'a str> for GenericProperty {
    fn into(self) -> &'a str {
        match self {
            GenericProperty::REG => "reg",
        }
    }
}

impl Display for DtbParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InvalidAddr(x) => write!(f, "invalid addr : {:#X}", x),
            DtbParseError::ParseError => write!(f, "parse error"),
        }
    }
}

impl<'a> DtbWrapper<'a> {
    pub fn parse(addr: usize) -> Result<DtbWrapper<'a>, DtbParseError> {
        let dtb = unsafe { Dtb::from_raw(addr as *const u8) }.ok_or(InvalidAddr(addr))?;
        Ok(Self { dtb })
    }

    /// 获取 memory 节点数据
    pub fn memory_node(&self) -> (usize, usize) {
        match self
            .dtb
            .enum_subnodes(ROOT.into())
            .find(|&node| node.starts_with::<&str>(MEMORY.into()))
        {
            None => (0, 0),
            Some(name) => self.parse_reg(&name),
        }
    }

    /// 获取 virtio_mmio 节点数据
    pub fn virtio_mmio_node(&self) -> Vec<(usize, usize)> {
        return match self
            .dtb
            .enum_subnodes(ROOT.into())
            .find(|&node| node.starts_with::<&str>(SOC.into()))
        {
            None => Vec::new(),
            Some(node) => self
                .dtb
                .enum_subnodes(node)
                .filter(|&node| node.starts_with::<&str>(VIRTIO_MMIO.into()))
                .map(|node| self.parse_reg([SOC.into(), node].join("/").as_str()))
                .collect::<Vec<(usize, usize)>>(),
        };
    }

    /// 获取指定节点的 reg 属性值
    fn parse_reg(&self, path: &str) -> (usize, usize) {
        match self.dtb.get_property(path, GenericProperty::REG.into()) {
            None => (0, 0),
            Some(reg) => (bebytes2usize(&reg[..8]), bebytes2usize(&reg[8..])),
        }
    }
}

/// 大端字节序数组转 usize
fn bebytes2usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.to_owned().try_into().unwrap())
}
