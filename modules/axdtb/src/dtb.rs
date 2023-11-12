use crate::dtb::DtbParseError::InvalidAddr;
use crate::dtb::GenericNodePrefix::{Memory, Root, Soc, VirtioMmio};
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
    Root,
    Memory,
    Soc,
    VirtioMmio,
}

impl<'a> Into<&'a str> for GenericNodePrefix {
    fn into(self) -> &'a str {
        match self {
            Root => "/",
            Memory => "memory",
            Soc => "soc",
            VirtioMmio => "virtio_mmio",
        }
    }
}

pub enum GenericProperty {
    Reg,
}

impl<'a> Into<&'a str> for GenericProperty {
    fn into(self) -> &'a str {
        match self {
            GenericProperty::Reg => "reg",
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
            .enum_subnodes(Root.into())
            .find(|&node| node.starts_with::<&str>(Memory.into()))
        {
            None => (0, 0),
            Some(name) => self.parse_reg(&name),
        }
    }

    /// 获取 virtio_mmio 节点数据
    pub fn virtio_mmio_node(&self) -> Vec<(usize, usize)> {
        return match self
            .dtb
            .enum_subnodes(Root.into())
            .find(|&node| node.starts_with::<&str>(Soc.into()))
        {
            None => Vec::new(),
            Some(node) => self
                .dtb
                .enum_subnodes(node)
                .filter(|&node| node.starts_with::<&str>(VirtioMmio.into()))
                .map(|node| self.parse_reg([Soc.into(), node].join("/").as_str()))
                .collect::<Vec<(usize, usize)>>(),
        };
    }

    /// 获取指定节点的 reg 属性值
    fn parse_reg(&self, path: &str) -> (usize, usize) {
        match self.dtb.get_property(path, GenericProperty::Reg.into()) {
            None => (0, 0),
            Some(reg) => (bebytes2usize(&reg[..8]), bebytes2usize(&reg[8..])),
        }
    }
}

/// 大端字节序数组转 usize
fn bebytes2usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.to_owned().try_into().unwrap())
}
