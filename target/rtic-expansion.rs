#[allow(non_snake_case)] fn init(mut cx : init :: Context) -> init ::
LateResources
{
    cx . core . DCB . enable_trace() ; cx . core . DWT .
    enable_cycle_counter() ; rtic :: pend(stm32f411 :: Interrupt :: EXTI1) ;
    init :: LateResources { dwt : cx . core . DWT }
} #[allow(non_snake_case)] fn exti0(cx : exti0 :: Context)
{ use rtic :: Mutex as _ ; asm :: bkpt() ; * cx . resources . shared += 1 ; }
#[allow(non_snake_case)] fn exti1(mut cx : exti1 :: Context)
{
    use rtic :: Mutex as _ ; unsafe
    { cx . resources . dwt . cyccnt . write(0) } ; asm :: bkpt() ; rtic ::
    pend(stm32f411 :: Interrupt :: EXTI0) ; asm :: bkpt() ; cx . resources .
    shared .
    lock(| shared | { asm :: bkpt() ; * shared += 1 ; asm :: bkpt() ; }) ; asm
    :: bkpt() ;
} #[doc = r" Resources initialized at runtime"] #[allow(non_snake_case)] pub
struct initLateResources { pub dwt : DWT } #[allow(non_snake_case)]
#[doc = "Initialization function"] pub mod init
{
    #[doc(inline)] pub use super :: initLateResources as LateResources ;
    #[doc = r" Execution context"] pub struct Context < >
    {
        #[doc = r" Core (Cortex-M) peripherals"] pub core : rtic :: export ::
        Peripherals,
    } impl < > Context < >
    {
        #[inline(always)] pub unsafe fn
        new(core : rtic :: export :: Peripherals,) -> Self
        { Context { core, } }
    }
} mod resources
{
    use rtic :: export :: Priority ; #[allow(non_camel_case_types)] pub struct
    shared < 'a > { priority : & 'a Priority, } impl < 'a > shared < 'a >
    {
        #[inline(always)] pub unsafe fn new(priority : & 'a Priority) -> Self
        { shared { priority } } #[inline(always)] pub unsafe fn
        priority(& self) -> & Priority { self . priority }
    }
} #[allow(non_snake_case)] #[doc = "Resources `exti0` has access to"] pub
struct exti0Resources < 'a > { pub shared : & 'a mut u64, }
#[allow(non_snake_case)] #[doc = "Hardware task"] pub mod exti0
{
    #[doc(inline)] pub use super :: exti0Resources as Resources ;
    #[doc = r" Execution context"] pub struct Context < 'a >
    {
        #[doc = r" Resources this task has access to"] pub resources :
        Resources < 'a >,
    } impl < 'a > Context < 'a >
    {
        #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        { Context { resources : Resources :: new(priority), } }
    }
} #[allow(non_snake_case)] #[doc = "Resources `exti1` has access to"] pub
struct exti1Resources < 'a >
{ pub dwt : & 'a mut DWT, pub shared : resources :: shared < 'a >, }
#[allow(non_snake_case)] #[doc = "Hardware task"] pub mod exti1
{
    #[doc(inline)] pub use super :: exti1Resources as Resources ;
    #[doc = r" Execution context"] pub struct Context < 'a >
    {
        #[doc = r" Resources this task has access to"] pub resources :
        Resources < 'a >,
    } impl < 'a > Context < 'a >
    {
        #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        { Context { resources : Resources :: new(priority), } }
    }
} #[doc = r" Implementation details"] const APP : () =
{
    #[doc =
      r" Always include the device crate which contains the vector table"] use
    stm32f411 as _ ; #[cfg(core = "1")] compile_error !
    ("specified 1 core but tried to compile for more than 1 core") ;
    #[allow(non_upper_case_globals)] static mut shared : u64 = 0 ; impl < 'a >
    rtic :: Mutex for resources :: shared < 'a >
    {
        type T = u64 ; #[inline(always)] fn lock < R >
        (& mut self, f : impl FnOnce(& mut u64) -> R) -> R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 2u8 ; unsafe
            {
                rtic :: export ::
                lock(& mut shared, self . priority(), CEILING, stm32f411 ::
                     NVIC_PRIO_BITS, f,)
            }
        }
    } #[allow(non_upper_case_globals)] #[link_section = ".uninit.rtic0"]
    static mut dwt : core :: mem :: MaybeUninit < DWT > = core :: mem ::
    MaybeUninit :: uninit() ; #[allow(non_snake_case)] #[no_mangle] unsafe fn
    EXTI0()
    {
        const PRIORITY : u8 = 2u8 ; rtic :: export ::
        run(PRIORITY, ||
            {
                crate ::
                exti0(exti0 :: Context ::
                      new(& rtic :: export :: Priority :: new(PRIORITY)))
            }) ;
    } impl < 'a > exti0Resources < 'a >
    {
        #[inline(always)] unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        { exti0Resources { shared : & mut shared, } }
    } #[allow(non_snake_case)] #[no_mangle] unsafe fn EXTI1()
    {
        const PRIORITY : u8 = 1u8 ; rtic :: export ::
        run(PRIORITY, ||
            {
                crate ::
                exti1(exti1 :: Context ::
                      new(& rtic :: export :: Priority :: new(PRIORITY)))
            }) ;
    } impl < 'a > exti1Resources < 'a >
    {
        #[inline(always)] unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            exti1Resources
            {
                dwt : & mut * dwt . as_mut_ptr(), shared : resources :: shared
                :: new(priority),
            }
        }
    } #[no_mangle] unsafe extern "C" fn main() -> !
    {
        let _TODO : () = () ; rtic :: export :: assert_send :: < DWT > () ;
        rtic :: export :: interrupt :: disable() ; let mut core : rtic ::
        export :: Peripherals = rtic :: export :: Peripherals :: steal() .
        into() ; let _ =
        [() ; ((1 << stm32f411 :: NVIC_PRIO_BITS) - 2u8 as usize)] ; core .
        NVIC .
        set_priority(stm32f411 :: Interrupt :: EXTI0, rtic :: export ::
                     logical2hw(2u8, stm32f411 :: NVIC_PRIO_BITS),) ; rtic ::
        export :: NVIC :: unmask(stm32f411 :: Interrupt :: EXTI0) ; let _ =
        [() ; ((1 << stm32f411 :: NVIC_PRIO_BITS) - 1u8 as usize)] ; core .
        NVIC .
        set_priority(stm32f411 :: Interrupt :: EXTI1, rtic :: export ::
                     logical2hw(1u8, stm32f411 :: NVIC_PRIO_BITS),) ; rtic ::
        export :: NVIC :: unmask(stm32f411 :: Interrupt :: EXTI1) ; core . SCB
        . scr . modify(| r | r | 1 << 1) ; let late = crate ::
        init(init :: Context :: new(core . into())) ; dwt . as_mut_ptr() .
        write(late . dwt) ; rtic :: export :: interrupt :: enable() ; loop
        { rtic :: export :: wfi() }
    }
} ;