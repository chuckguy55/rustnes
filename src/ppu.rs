use std::mem;

/// # Memory Map
/// This is from http://nesdev.com/NESDoc.pdf
///  ___________________ $10000  ________________
/// | Mirrors           |       | Mirrors        |
/// | $0000-$3FFF       |       | $0000-$3FFF    |
/// |___________________| $4000 |________________|
/// | Mirrors           |       |                |
/// | $3F00-$3F1F       |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $3F20 |                |
/// | Sprite Palette    |       | Palettes       |
/// |_ _ _ _ _ _ _ _ _ _| $3F10 |                |
/// | Image Palette     |       |                |
/// |___________________| $3F00 |________________|
/// | Mirrors           |       |                |
/// | $2000-$2EFF       |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $3000 |                |
/// | Attribute Table 3 |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $2FC0 |                |
/// | Name Table 3      |       |                |
/// |___________________| $2C00 |                |
/// | Attribute Table 2 |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $2BC0 |                |
/// | Name Table 2      |       | Name Tables    |
/// |___________________| $2800 |                |
/// | Attribute Table 1 |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $27C0 |                |
/// | Name Table 1      |       |                |
/// |___________________| $2400 |                |
/// | Attribute Table 0 |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $23C0 |                |
/// | Name Table 0      |       |                |
/// |___________________| $2000 |________________|
/// | Pattern Table 1   |       |                |
/// |_ _ _ _ _ _ _ _ _ _| $1000 | Pattern Tables |
/// | Pattern Table 0   |       |                |
/// |___________________| $0000 |________________|
///
///
/// # PPU Registers
///
/// This is from http://nesdev.com/NESDoc.pdf
///
/// $2000 - PPU Control Register 1 - Write Only
///
/// - Bits 1-0 -  Name Table address, changes between the four
///   name tables at $2000 (0b00), $2400 (0b01), $2800 (0b10) and $2C00 (0b11).
///
/// - Bit 2 - Specifies the amout to increment address by, either 1 if 
///   this is 0 or 32 if this is 1.
///
/// - Bit 3 - Identifies which pattern table Sprites are stored in,
///   either $0000 (0) or $1000 (1).
///
/// - Bit 4 - Identifies which pattern table the background 
///   is store in, either $0000 (0) or $1000 (1).
///
/// - Bit 5 - Specifies the size of sprites in pixels, 8x8 
///   if this is 0, 8x16 if this is 1.
///
/// - Bit 6 - Changes PPU between master and slave modes. 
///   This is not used by the NES.
///
/// - Bit 7 - Indicates whether a NMI should occur up V-Blank.
///
/// $2001 - PPU Control Register 2 - Write Only
///
/// - Bit 0 - Indicates whether the system is in color (0) or 
///   monochrome mode (1).
///
/// - Bit 1 - Specifies whether to clip the background, that is whether
///   to hide the background in the left 8 pixels on screen (0) or 
///   or to show them (1).
///
/// - Bit 2 - Specifies whether to clip the sprites, that is whether
///   to hide sprites in the left 8 pixels on screen (0) or 
///   or to show them (1).
///
/// - Bit 3 - If this is 0, the background should not be displayed
///
/// - Bit 4 - If this is 0, sprites should not be dispalyed
///
/// - Bits 7-5 - Indicates background color in monochrome mode or 
///   color intensity in color mode.
///
/// $2002 - PPU Status Register - Read Only
///
/// - Bit 4 - If set, indicates that writes to VRAM should be ignored.
///
/// - Bit 5 - Scanline sprite count, if set, indicates more than 8 
///   sprites on the current scanline.
///
/// - Bit 6 - Sprite 0 hit flag, set when a non-transparent pixel of
///   sprite 0 overlaps a non-transparent background pixel
///
/// - Bit 7 - Indicates whether V-Blank is occurring.
///
/// $2003 - SprRam VAddr Register - Write Only
///
/// - Holds the VAddr in SprRam to access on the next write to $2004
///
/// $2004 - SprRam I/O Register - Write Only
///
/// - Writes a byte to SprRam at the VAddr indicated by $2003
///
/// $2005 - VRAM Address Register 1 - Write Only
///
/// $2006 - VRAM Address Register 2 - Write Only
///
/// $2007 - VRAM I/O Register - Read/Write
///
/// - Reads or writes a byte from VRAM at the current address.
///
/// TODO
/// DMA Register ($4014) and Joypad I/O Registers ($4016 and $4017)
///





/// # Sprites
///
/// This is from http://nesdev.com/NESDoc.pdf but
/// better doc can be found at http://wiki.nesdev.com/w/index.php/PPU_OAM
///
/// - Byte 0 - Stores the Y coordinate of the top of the sprite minus 1
/// - Byte 1 - Index number of the sprite in the patter tables
/// - Byte 2 - Stores the attributes of the sprites
/// -- Bits 1-0 - Most signifigant bits of the color
/// -- Bit 5    - Indicates whether this sprite has priority over the background
/// -- Bit 6    - Indicates whether to flip the sprite horizontally
/// -- Bit 7    - Indicates whether to flip the sprite vetically
/// - Byte 3 - Stores the X coordinate of the left of the sprite
/// -- X-scroll values of F9-FF do NOT result in the sprite wrapping 
///    around to the left side of the screen.

bitflags!(
    flags SprAttr: u8 {
        static COLOR_MASK    = 0b00000011,
        static PRIORITY_FLAG = 0b00100000,
        static H_FLIP        = 0b01000000,
        static V_FLIP        = 0b10000000
    }
)

struct Spr {
    Y: u8,
    I: u8,
    attr: SprAttr,
    X: u8,
}

impl Spr {
    //make a Spr out of 4 bytes
    #[inline]
    pub fn new(bytes: [u8, ..4]) -> Spr {
        let spr: &Spr;
        unsafe { spr = mem::transmute(bytes.as_ptr()) }
        *spr
    }

    #[inline]
    //returns the correctly alligned color bits for a pallete lookup
    //i.e. if attr = 0b00000011 then this returns 0b00001100
    pub fn color(&self) -> u8 {
        (self.attr & COLOR_MASK).bits << 2
    }

    #[inline]
    pub fn has_priority(&self) -> bool {
        self.attr.contains(PRIORITY_FLAG)
    }

    #[inline]
    pub fn h_flip(&self) -> bool {
        self.attr.contains(H_FLIP)
    }

    #[inline]
    pub fn v_flip(&self) -> bool {
        self.attr.contains(V_FLIP)
    }
}
