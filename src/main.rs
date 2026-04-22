#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::{
    I2c,
    prelude::*,
    hal::port::{PC4, PC5},
    port::{mode::Input, Pin},
};

/// LCD 控制器結構（物件化封裝）
/// LCD Controller Struct (OOP Encapsulation)
struct Lcd1602I2c<I2C> {
    i2c: I2C,
    address: u8,
    backlight: u8,
}

/// LCD 指令列舉
/// LCD Command Enum
#[repr(u8)]
#[allow(dead_code)]
enum LcdCommand {
    ClearDisplay = 0x01,
    ReturnHome = 0x02,
    EntryModeSet = 0x06,
    DisplayOn = 0x0C,
    FunctionSet = 0x28,  // 4-bit, 2-line, 5x8 dots
    SetDdramAddr = 0x80,
}

/// LCD 控制位元
/// LCD Control Bits (PCF8574 pin mapping)
const LCD_BACKLIGHT: u8 = 0x08;
const LCD_EN: u8 = 0x04;        // Enable bit
const LCD_RS: u8 = 0x01;        // Register Select (0=command, 1=data)

impl<I2C, E> Lcd1602I2c<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    /// 建構子：初始化 LCD 結構
    /// Constructor: Initialize LCD struct
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            backlight: LCD_BACKLIGHT,
        }
    }

    /// 初始化 LCD（HD44780 官方初始化序列）
    /// Initialize LCD (official HD44780 init sequence)
    pub fn init(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) -> Result<(), E> {
        delay.delay_ms(50);

        // 三次 0x30 → 進入 4-bit 模式
        self.write_nibble(0x30, delay)?;
        delay.delay_ms(5);
        self.write_nibble(0x30, delay)?;
        delay.delay_us(150);
        self.write_nibble(0x30, delay)?;
        self.write_nibble(0x20, delay)?;  // 切換至 4-bit

        self.send_command(LcdCommand::FunctionSet as u8, delay)?;
        self.send_command(LcdCommand::DisplayOn as u8, delay)?;
        self.send_command(LcdCommand::ClearDisplay as u8, delay)?;
        delay.delay_ms(2);
        self.send_command(LcdCommand::EntryModeSet as u8, delay)?;
        Ok(())
    }

    /// 送出指令
    /// Send command byte
    pub fn send_command(
        &mut self,
        cmd: u8,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        self.write_byte(cmd, 0, delay)
    }

    /// 送出資料（字元）
    /// Send data byte (character)
    pub fn send_data(
        &mut self,
        data: u8,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        self.write_byte(data, LCD_RS, delay)
    }

    /// 寫入字串
    /// Write string to LCD
    pub fn write_str(
        &mut self,
        s: &str,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        for c in s.bytes() {
            self.send_data(c, delay)?;
        }
        Ok(())
    }

    /// 設定游標位置
    /// Set cursor position (col: 0-15, row: 0-1)
    pub fn set_cursor(
        &mut self,
        col: u8,
        row: u8,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        let addr = if row == 0 { 0x00 } else { 0x40 } + col;
        self.send_command(LcdCommand::SetDdramAddr as u8 | addr, delay)
    }

    /// 清除螢幕
    /// Clear display
    pub fn clear(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) -> Result<(), E> {
        self.send_command(LcdCommand::ClearDisplay as u8, delay)?;
        delay.delay_ms(2);
        Ok(())
    }

    // ─── 私有輔助方法 ───

    fn write_byte(
        &mut self,
        byte: u8,
        mode: u8,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        let high_nibble = byte & 0xF0;
        let low_nibble = (byte << 4) & 0xF0;
        self.write_nibble(high_nibble | mode, delay)?;
        self.write_nibble(low_nibble | mode, delay)?;
        Ok(())
    }

    fn write_nibble(
        &mut self,
        nibble: u8,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), E> {
        let data = nibble | self.backlight;
        // 拉高 EN
        self.i2c.write(self.address, &[data | LCD_EN])?;
        delay.delay_us(1);
        // 拉低 EN（資料鎖存）
        self.i2c.write(self.address, &[data & !LCD_EN])?;
        delay.delay_us(50);
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // 初始化 I2C（A4=SDA, A5=SCL, 100kHz）
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // ─── 背光硬體測試 ───
    // 只送背光 bit（0x08）到 PCF8574，觀察 2 秒：
    //   亮   → 背光硬體 OK，問題在後續 init 序列
    //   不亮 → 背光 LED / 電晶體 / 跳線有問題（需排查硬體）
    // 使用完全限定路徑以避開 arduino_hal::prelude 內多個 write trait 的歧義
    //embedded_hal::i2c::I2c::write(&mut i2c, 0x27, &[0x08]).ok();
    //arduino_hal::delay_ms(2000);

    // 建立 LCD 物件（I2C 地址通常為 0x27 或 0x3F）
    let mut lcd = Lcd1602I2c::new(i2c, 0x27);
    lcd.init(&mut delay).ok();

    // 顯示訊息
    lcd.set_cursor(0, 0, &mut delay).ok();
    lcd.write_str("Hello, Mary!", &mut delay).ok();

    lcd.set_cursor(0, 1, &mut delay).ok();
    lcd.write_str("Arduino Uno", &mut delay).ok();

    loop {
        arduino_hal::delay_ms(1000);
    }
}
