#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt;

macro_rules! define_interfaces {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident = $id:expr,
        )+
    ) => {
        /// List of target interfaces.
        ///
        /// Note that this library might not support all of them, despite listing them here.
        #[non_exhaustive]
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum Interface {
            $(
                $( #[$attr] )*
                $name = $id,
            )+
        }

        impl Interface {
            const ALL: &'static [Self] = &[
                $( Self::$name ),+
            ];

            pub(crate) fn from_u32(raw: u32) -> Option<Self> {
                match raw {
                    $(
                        $id => Some(Self::$name),
                    )+
                    _ => None,
                }
            }
        }

        bitflags! {
            struct InterfaceFlags: u32 {
                $(
                    const $name = 1 << $id;
                )+
            }
        }
    };
}

define_interfaces!(
    /// JTAG interface (IEEE 1149.1). Supported by all J-Link probes.
    Jtag = 0,
    /// SWD interface (Serial Wire Debug), used by most Cortex-M chips, and supported by almost all
    /// J-Link probes.
    Swd = 1,
    /// Background Debug Mode 3, a single-wire debug interface used on some NXP microcontrollers.
    Bdm3 = 2,
    /// FINE, a two-wire debugging interface used by Renesas RX MCUs.
    // FIXME: There's a curious bug that hangs the probe when selecting the FINE interface.
    // Specifically, the probe never sends back the previous interface after it receives the `c7 03`
    // SELECT_IF cmd, even though the normal J-Link software also just sends `c7 03` and gets back
    // the right response.
    Fine = 3,
    /// In-Circuit System Programming (ICSP) interface of PIC32 chips.
    Pic32Icsp = 4,
    /// Serial Peripheral Interface (for SPI Flash programming).
    Spi = 5,
    /// Silicon Labs' 2-wire debug interface.
    C2 = 6,
    /// [cJTAG], or compact JTAG, as specified in IEEE 1149.7.
    ///
    /// [cJTAG]: https://wiki.segger.com/J-Link_cJTAG_specifics.
    CJtag = 7,
    /// 2-wire debugging interface used by Microchip's IS208x MCUs.
    Mc2WireJtag = 10,
    // (*)
    // NOTE: When changing this enum, also change all other places with a (*) in addition to
    // anything that fails to compile.
    // NOTE 2: Keep the docs in sync with the bitflags below!
);

impl Interface {
    pub(crate) fn as_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Interface::Jtag => "JTAG",
            Interface::Swd => "SWD",
            Interface::Bdm3 => "BDM3",
            Interface::Fine => "FINE",
            Interface::Pic32Icsp => "PIC32 ICSP",
            Interface::Spi => "SPI",
            Interface::C2 => "C2",
            Interface::CJtag => "cJTAG",
            Interface::Mc2WireJtag => "Microchip 2-wire JTAG",
        })
    }
}

/// A set of supported target interfaces.
///
/// This implements `IntoIterator`, so you can call `.into_iter()` to iterate over the contained
/// [`Interface`]s.
///
/// [`Interface`]: enum.Interface.html
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Interfaces(InterfaceFlags);

impl Interfaces {
    pub(crate) fn from_bits_warn(raw: u32) -> Self {
        let flags = InterfaceFlags::from_bits_truncate(raw);
        if flags.bits() != raw {
            log::debug!(
                "unknown bits in interface mask: 0x{:08X} truncated to 0x{:08X} ({:?})",
                raw,
                flags.bits(),
                flags,
            );
        }
        Self(flags)
    }

    /// Returns whether `interface` is contained in `self`.
    pub fn contains(&self, interface: Interface) -> bool {
        self.0
            .contains(InterfaceFlags::from_bits(1 << interface as u32).unwrap())
    }
}

impl fmt::Debug for Interfaces {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl IntoIterator for Interfaces {
    type Item = Interface;
    type IntoIter = InterfaceIter;

    fn into_iter(self) -> Self::IntoIter {
        InterfaceIter {
            interfaces: self,
            next: 0,
        }
    }
}

/// Iterator over supported `Interface`s.
#[derive(Debug)]
pub struct InterfaceIter {
    interfaces: Interfaces,
    next: usize,
}

impl Iterator for InterfaceIter {
    type Item = Interface;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = Interface::ALL.get(self.next)?;
            self.next += 1;
            if self.interfaces.contains(*next) {
                return Some(*next);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        assert_eq!(
            Interfaces(InterfaceFlags::empty())
                .into_iter()
                .collect::<Vec<_>>(),
            &[]
        );
        assert_eq!(
            Interfaces(InterfaceFlags::Jtag)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Interface::Jtag]
        );
        assert_eq!(
            Interfaces(InterfaceFlags::Swd)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Interface::Swd]
        );
        assert_eq!(
            Interfaces(InterfaceFlags::Jtag | InterfaceFlags::Swd)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Interface::Jtag, Interface::Swd]
        );
    }
}
