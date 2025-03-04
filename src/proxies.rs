use embedded_hal::adc;
use embedded_hal::blocking::i2c;
use embedded_hal::blocking::spi;
use embedded_hal::blocking::can;

/// Proxy type for I2C bus sharing.
///
/// The `I2cProxy` implements all (blocking) I2C traits so it can be passed to drivers instead of
/// the bus instance.  Internally, it holds reference to the bus via a mutex, ensuring that all
/// accesses are strictly synchronized.
///
/// An `I2cProxy` is created by calling [`BusManager::acquire_i2c()`][acquire_i2c].
///
/// [acquire_i2c]: ./struct.BusManager.html#method.acquire_i2c
#[derive(Debug)]
pub struct I2cProxy<'a, M> {
    pub(crate) mutex: &'a M,
}

impl<'a, M: crate::BusMutex> Clone for I2cProxy<'a, M> {
    fn clone(&self) -> Self {
        Self { mutex: &self.mutex }
    }
}

impl<'a, M: crate::BusMutex> i2c::Write for I2cProxy<'a, M>
where
    M::Bus: i2c::Write,
{
    type Error = <M::Bus as i2c::Write>::Error;

    fn write(&mut self, addr: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        self.mutex.lock(|bus| bus.write(addr, buffer))
    }
}

impl<'a, M: crate::BusMutex> i2c::Read for I2cProxy<'a, M>
where
    M::Bus: i2c::Read,
{
    type Error = <M::Bus as i2c::Read>::Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.mutex.lock(|bus| bus.read(addr, buffer))
    }
}

impl<'a, M: crate::BusMutex> i2c::WriteRead for I2cProxy<'a, M>
where
    M::Bus: i2c::WriteRead,
{
    type Error = <M::Bus as i2c::WriteRead>::Error;

    fn write_read(
        &mut self,
        addr: u8,
        buffer_in: &[u8],
        buffer_out: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.mutex
            .lock(|bus| bus.write_read(addr, buffer_in, buffer_out))
    }
}

/// Proxy type for SPI bus sharing.
///
/// The `SpiProxy` implements all (blocking) SPI traits so it can be passed to drivers instead of
/// the bus instance.  An `SpiProxy` is created by calling [`BusManager::acquire_spi()`][acquire_spi].
///
/// **Note**: The `SpiProxy` can only be used for sharing **withing a single task/thread**.  This
/// is due to drivers usually managing the chip-select pin manually which would be inherently racy
/// in a concurrent environment (because the mutex is locked only after asserting CS).  To ensure
/// safe usage, a `SpiProxy` can only be created when using [`BusManagerSimple`] and is `!Send`.
///
/// [acquire_spi]: ./struct.BusManager.html#method.acquire_spi
/// [`BusManagerSimple`]: ./type.BusManagerSimple.html
#[derive(Debug)]
pub struct SpiProxy<'a, M> {
    pub(crate) mutex: &'a M,
    pub(crate) _u: core::marker::PhantomData<*mut ()>,
}

impl<'a, M: crate::BusMutex> Clone for SpiProxy<'a, M> {
    fn clone(&self) -> Self {
        Self {
            mutex: &self.mutex,
            _u: core::marker::PhantomData,
        }
    }
}

impl<'a, M: crate::BusMutex> spi::Transfer<u8> for SpiProxy<'a, M>
where
    M::Bus: spi::Transfer<u8>,
{
    type Error = <M::Bus as spi::Transfer<u8>>::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.mutex.lock(move |bus| bus.transfer(words))
    }
}

impl<'a, M: crate::BusMutex> spi::Write<u8> for SpiProxy<'a, M>
where
    M::Bus: spi::Write<u8>,
{
    type Error = <M::Bus as spi::Write<u8>>::Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.mutex.lock(|bus| bus.write(words))
    }
}

/// Proxy type for ADC sharing.
///
/// The `AdcProxy` implements OneShot trait so it can be passed to drivers instead of
/// ADC instance. Internally, it holds reference to the bus via a mutex, ensuring
/// that all accesses are strictly synchronized.
///
/// An `AdcProxy` is created by calling [`BusManager::acquire_adc()`][acquire_adc].
///
/// **Note**: The [`adc::OneShot`] trait proxied by this type describes a
/// non-blocking contract for ADC read operation.  However access to a shared ADC
/// unit can not be arbitrated in a completely non-blocking and concurrency safe way.
/// Any reading from a channel shall be completed before `shared-bus` can allow the
/// next read from the same or another channel. So the current implementation breaks
/// the non-blocking contract of the trait and just busy-spins until a sample is
/// returned.
///
/// [acquire_adc]: ./struct.BusManager.html#method.acquire_adc
#[derive(Debug)]
pub struct AdcProxy<'a, M> {
    pub(crate) mutex: &'a M,
}

impl<'a, M: crate::BusMutex> Clone for AdcProxy<'a, M> {
    fn clone(&self) -> Self {
        Self { mutex: &self.mutex }
    }
}

impl<'a, M: crate::BusMutex, ADC, Word, Pin> adc::OneShot<ADC, Word, Pin> for AdcProxy<'a, M>
where
    Pin: adc::Channel<ADC>,
    M::Bus: adc::OneShot<ADC, Word, Pin>,
{
    type Error = <M::Bus as adc::OneShot<ADC, Word, Pin>>::Error;

    fn read(&mut self, pin: &mut Pin) -> nb::Result<Word, Self::Error> {
        self.mutex
            .lock(|bus| nb::block!(bus.read(pin)).map_err(nb::Error::Other))
    }
}

/// Proxy type for CAN bus sharing.
///
/// The `CanProxy` implements all (blocking) CAN traits so it can be passed to drivers instead of
/// the bus instance.  Internally, it holds reference to the bus via a mutex, ensuring that all
/// accesses are strictly synchronized.
///
/// A `CanProxy` is created by calling [`BusManager::acquire_can()`][acquire_can].
///
/// [acquire_can]: ./struct.BusManager.html#method.acquire_can

#[derive(Debug)]
pub struct CanProxy<'a, M> {
    pub(crate) mutex: &'a M,
}

impl<'a, M: crate::BusMutex> Clone for CanProxy<'a, M> {
    fn clone(&self) -> Self {
        Self { mutex: &self.mutex }
    }
}

impl<'a, M: crate::BusMutex> can::Can for CanProxy<'a, M>
where
    M::Bus: can::Can,
{
    type Error = <M::Bus as can::Can>::Error;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        self.mutex.lock(|bus| bus.transmit(frame))
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        self.mutex.lock(|bus| bus.receive())
    }

    type Frame = <M::Bus as embedded_hal::blocking::can::Can>::Frame;
}