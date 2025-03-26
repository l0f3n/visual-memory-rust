use core::cell::RefCell;

pub type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
pub static CONSOLE: avr_device::interrupt::Mutex<RefCell<Option<Console>>> =
    avr_device::interrupt::Mutex::new(RefCell::new(None));

#[allow(unused_macros)]
macro_rules! print {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(console) = crate::print::CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwrite!(console, $($t)*);
                }
            },
        )
    };
}

#[allow(unused_macros)]
#[macro_export] macro_rules! println {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(console) = crate::print::CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            }
        )
    }
}

pub fn put_console(console: Console) {
    avr_device::interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}

// pub fn get_type_name<T>(_: T) -> &'static str {
//     nostd::any::type_name::<T>()
// }
// pub fn print_type_name<T>(_: T) {
//     let type_name = nostd::any::type_name::<T>();
//     println!("{}", type_name);
// }
// 
