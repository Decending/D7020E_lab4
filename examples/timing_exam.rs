//! examples/timing_exam.rs

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{asm, peripheral::DWT};
use panic_halt as _;
use rtic::cyccnt::{Duration, Instant, U32Ext};
use stm32f4::stm32f411;

#[no_mangle]
static mut T1_MAX_RP: u32 = 0;
#[no_mangle]
static mut T2_MAX_RP: u32 = 0;
#[no_mangle]
static mut T3_MAX_RP: u32 = 0;

#[rtic::app(device = stm32f411, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        #[init(0)]
        R1: u64, // non atomic data
        #[init(0)]
        R2: u64, // non atomic data
    }

    #[init(schedule = [t1, t2, t3])]
    fn init(mut cx: init::Context) {
        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();
        cx.schedule.t1(cx.start + 100_000.cycles()).unwrap();
        cx.schedule.t2(cx.start + 200_000.cycles()).unwrap();
        cx.schedule.t3(cx.start + 50_000.cycles()).unwrap();
    }

    // Deadline 100, Inter-arrival 100
    #[inline(never)]
    #[task(schedule = [t1], priority = 1)]
    fn t1(cx: t1::Context) {
        asm::bkpt();
        cx.schedule.t1(cx.scheduled + 100_000.cycles()).unwrap();
        asm::bkpt();

        // emulates timing behavior of t1
        let start = DWT::get_cycle_count();
        cortex_m::asm::delay(10_000); // no locks
        let end = DWT::get_cycle_count();
        asm::bkpt();

        // 2) your code here to update T1_MAX_RP and
        // break if deadline missed
        unsafe{
            if (end - start) > T1_MAX_RP{
                T1_MAX_RP = end.wrapping_sub(start);
            }
            if (end - start) > 100_000{
               asm::bkpt();
            }
        }
    }

    // Deadline 200, Inter-arrival 200
    #[inline(never)]
    #[task(schedule = [t2], resources = [R1, R2], priority = 2)]
    fn t2(cx: t2::Context) {
        asm::bkpt();
        cx.schedule.t2(cx.scheduled + 200_000.cycles()).unwrap();
        asm::bkpt();

        // 1) your code here to emulate timing behavior of t2
        let start = DWT::get_cycle_count();
        cortex_m::asm::delay(10_000); // 0-10
        cortex_m::asm::delay(2_000);  // 10-12, r1 is locked at 10
        cortex_m::asm::delay(4_000); // 12-16, this is where r2 is locked and unlocked
        cortex_m::asm::delay(4_000); // 16-20, r1 is unlocked at 20
        cortex_m::asm::delay(2_000); // 20-22
        cortex_m::asm::delay(6_000); // 22-28, r1 is locked between 22 and 28
        cortex_m::asm::delay(2_000); // 28-30
        let end = DWT::get_cycle_count();
        asm::bkpt();

        // 2) your code here to update T2_MAX_RP and
        // break if deadline missed
        unsafe{
            if (end - start) > T2_MAX_RP{
                T2_MAX_RP = end.wrapping_sub(start);
            }
            if (end - start) > 200_000{
               asm::bkpt();
            }
        }
    }

    // Deadline 50, Inter-arrival 50
    #[inline(never)]
    #[task(schedule = [t3], resources = [R2], priority = 3)]
    fn t3(cx: t3::Context) {
        asm::bkpt();
        cx.schedule.t3(cx.scheduled + 50_000.cycles()).unwrap();
        asm::bkpt();

        // 1) your code here to emulate timing behavior of t3
        let start = DWT::get_cycle_count();
        cortex_m::asm::delay(10_000); // 0-10
        cortex_m::asm::delay(10_000); // 10-20, this is where r2 is locked and unlocked
        cortex_m::asm::delay(10_000); // 20-30
        let end = DWT::get_cycle_count();
        asm::bkpt();

        // 2) your code here to update T3_MAX_RP and
        // break if deadline missed
        unsafe{
            if (end - start) > T3_MAX_RP{
                T3_MAX_RP = end.wrapping_sub(start);
            }
            if (end - start) > 50_000{
               asm::bkpt();
            }
        }
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn EXTI0();
        fn EXTI1();
        fn EXTI2();
    }
};

// !!!! NOTICE !!!!
//
// Use either vscode with the `Cortex Nightly` launch profile,
// or compile with the feature `--features nightly` in order to
// get inlined assembly!
//
// 1) For this assignment you should first generate a task set that
// matches the example task set from `klee_tutorial/srp_analysis/main.rs`.
//
// Assume that each time unit amounts to 1_000 clock cycles, then
// the execution time of `t1` should be 10_000 clock cycles.
//
// So, instead of measuring execution time of an existing application,
// you are to create a task set according to given timing properties.
//
// Do this naively, by just calling `asm::delay(x)`, where x
// amounts to the number of clock cycles to spend.
//
// Commit your repository once your task set is implemented.
//
// 2) Code instrumentation:
// Now its time to see if your scheduling analysis is accurate
// in comparison to a real running system.
//
// First explain in your own words how the `Instant` is
// used to generate a periodic task instance arrivals.
//
// `cx.schedule.t1(cx.scheduled + 100_000.cycles()).unwrap();`
//
// [My answer here]
// By taking the latest scheduled time of the task and adding 100 000 clock cycles and iterrating it will become periodic with a period of 100 000 clock cycles
//
// Explain in your own words the difference between:
//
// `cx.schedule.t1(Instant::now() + 100_000.cycles()).unwrap();`
// and
// `cx.schedule.t1(cx.scheduled + 100_000.cycles()).unwrap();`
//
// [My answer here]
// Instant::now() takes the current "time" and schedules the task t1 100 000 clock cycles from there, making it hard to ensure exact periodicity.
// cx.scheduled takes the latest scheduled "time" and schedules the task t1 100 000 clock cycles from there, which ensures periodicity.
// Depending on how Instant::now() is used it will "work", but cx.scheduled is vastly easier and safer to implement.
// 
// Explain in your own words why we use the latter
// in order to generate a periodic task.
//
// [My answer here]
// It ensures that the "time" between the tasks are identical each time, while instant::now() may incur some inconsistencies
//
// Hint, look at https://rtic.rs/0.5/book/en/by-example/timer-queue.html
//
// Once you understand how `Instant` is used, document your crate:
// > cargo doc --open
//
// Once you have the documentation open, search for `Instant`
// Hint, you can search docs by pressing S.
//
// Now figure out how to calculate the actual response time.
// If the new response time is larger than the stored response time
// then update it (`T1_MAX_RP`, `T2_MAX_RP`, `T3_MAX_RP` respectively).
// If the response time is larger than the deadline, you should
// hit a `asm::bkpt()`, to indicate that an error occurred.
//
// You will need `unsafe` code to access the global variables.
//
// Explain why this is needed (there is a good reason for it).
//
// [My answer here]
// It's impossible to guarantee that there are no race conditions, in this context a data race. This makes the code inherently unsafe and hence we need to implement it in an unsafe way.
//
// Implement this functionality for all tasks.
//
// Commit your repository once you are done with the instrumentation.
//
// 3) Code Testing:
//
// Once the instrumentation code is in place, its finally time
// to test/probe/validate the system.
//
// Make sure that all tasks is initially scheduled from `init`.
//
// You can put WATCHES in vscode for the symbols
// WATCH
//  `T1_MAX_RP`
//  `T2_MAX_RP`
//  `T3_MAX_RP`
// To see them being updated during the test.
//
// The first breakpoint hit should be:
// fn t3(cx: t3::Context) {
//      asm::bkpt();
//
// Check the value of the CYCCNT register.
// (In vscode look under CORTEX PERIPHERALS > DWT > CYCCNT)
//
// Your values may differ slightly but should be in the same
// territory (if not, check your task implementation(s).)
//
// Task Entry Times, Task Nr, Response time Update
//   50240           t3       -
//                            30362
//  100295           t3
//                            30426
//
//  130595           t1
//
// At this point we can ask ourselves a number of
// interesting questions. Try answering in your own words.
//
// 3A) Why is there an offset 50240 (instead of 50000)?
//
// [My answer here]
// We're back in the real world now, so there will be some additional overhead.
//
// 3B) Why is the calculated response time larger than the
// delays you inserted to simulate workload?
//
// [My answer here]
// The overhead has to be accounted for.
// 
// 3C) Why is the second arrival of `t3` further delayed?
//
// [Your answer here]
// The switch from t1 to t3 will surely incur a certain extra delay, although it should be rather small.
//
// Hint, think about what happens at time 100_000, what tasks
// are set to `arrive` at that point compared to time 50_000.
//
// 3D) What is the scheduled time for task `t1` (130595 is the
// measured time according to CYCYCNT).
//
// [My answer here]
// It should be scheduled for 100 000
//
// Why is the measured value much higher than the scheduled time?
//
// [My answer here]
// Task t3 is scheduled at the same time slot and will preempt t1, which should add slightly more than 30 000 cycles.
//
// Now you can continue until you get a first update of `T1_MAX_RP`.
//
// What is the first update of `T1_MAX_RP`?
//
// [My answer here]
// Should land slightly above 40 000.
//
// Explain the obtained value in terms of:
// Execution time, blocking and preemptions
// (that occurred for this task instance).
//
// [My answer here]
// t1 will be preempted by t3, which will take slightly above 30 000 cycles. Slightly above due to the overhead and possible context switch from t1.
// t1 will also execute of course, so it adds another overhead and 10 000 cycles.
// The result should land somewhere slightly above 40 000.
//
// Now continue until you get a first timing measurement for `T2_MAX_RP`.
//
// What is the first update of `T2_MAX_RP`?
//
// [My answer here]
// All three tasks will be scheduled at 200 000:
// t3 will run first, as it has the higher priority: 230 000
// t2 will start running, become preempted by t3: 250 000 (10 000 left on t2)
// t3 wil run: 280 000
// t2 will continue, finishing at around 290 000.
// The first update of T2_MAX_RP should land slightly above 90 000.
//
// Now continue until you get a second timing measurement for `T1_MAX_RP`.
//
// What is the second update of `T1_MAX_RP`?
//
// [My answer here]
// Let's continue from the last answer:
// - We're at slightly above 290 000.
// - T1 was scheduled for 200 000, so it has been waiting for 90 000 cycles.
// Continuation:
// t1 will run from slightly above 290 000.
// !! Considering the overhead and the context switching, t3 will have time to preempt t1 at 300 000 !!
// t3 will run: 330 000
// t1 will finish: 330 000
// The second update of T1_MAX_RP will be slightly above 130 000, resulting in a deadline miss.
// 
// Now you should have ended up in a deadline miss right!!!!
//
// Why did this happen?
//
// [My answer here]
// The deadline miss will have to be due to all three tasks being scheduled at 200 000,
//
// Compare that to the result obtained from your analysis tool.
//
// Do they differ, if so why?
//
// [My answer here]
// Sadly, real life technology has the limitations of the real world and everything which comes with it.
// Things like the overhead, resource management and context switching takes clock cycles in real life and
// in our simulations that is not accounted for.
//     However, a few modifications to equation 7.22 actually accounts for the last execution of t3, which is missed
// in the provided equation due to how it rounds the answers.
//
// Commit your repository once you completed this part.
//
// 4) Delay tuning.
//
// So there were some discrepancy between the timing properties
// introduced by the `delay::asm` and the real measurements.
//
// Adjust delays to compensate for the OH to make it fit to
// to the theoretical task set.
//
// In order to do so test each task individually, schedule ony one
// task from `init` at a time.
//
// You may need to insert additional breakpoints to tune the timing.
//
// Once you are convinced that each task now adheres to
// the timing specification you can re-run part 3.
//
// If some task still misses its deadline go back and adjust
// the timing until it just passes.
//
// Commit your tuned task set.
//
// 5) Final remarks and learning outcomes.
//
// This exercise is of course a bit contrived, in the normal case
// you would start out with a real task set and then pass it
// onto analysis.
//
// Essay question:
//
// Reflect in your own words on:
//
// - RTIC and scheduling overhead
// - Coupling in between theoretical model and measurements
// - How would an ideal tool for static analysis of RTIC models look like.
//
// [My ideas and reflections here]
// - RTIC and scheduling overhead
// RTIC works with relatively small amounts of overhead, but while it is small we still cannot neglect it as shown in the above exercise.
//
// - Coupling in between theoretical models and measurements
// When we compare the theoretical models and the measurements we will find that the measurements will always be higher due to accounting for overhead / context switching / resource management, etc.
// However, while we can't calculate the exact amount of cycles we will need for the tasks, the worst case will always be a bounded value which we can calculate in our theoretical models
// relatively well.
//   My general thoughts are that that the theoretical models are useful as they are faster to make and can detect errors early in the development cycle, but it is not complete by itself and needs
// the precision of the measurements to fully prove that the code is sound.
// 
// - How would an ideal tool for static analysis of RTIC models look like
// At first I figured something which would inject the necessary code to count the clock cycles for each function and stop at that, but now I'm imagining something a bit more interactive.
// We should be able to inject the necessary code for clock cycle counting, but by doing that for the entire program would add a lot of unnecessary time to the debugging process if we know the
// area of the code which needs to be debugged, especially in larger projects, so we need something more versatile. 
//   By instead fetching the functions, each one disaplying the helper functions used as so:
//
// - Task 1
//   - Function call 1
//     - helper 1
//   - Function call 2
//     - helper 1
//
// and then being able to inject the necessary code for clock cycle counting, break points, etc, would give a more versitile tool which in the end would be both easier to use and more useful for
// the developers. This would also be more precise, as we might only be interested in helper 1, for example.
// One part of the tool should be fully automated for the things which are universally applicable to all tasks, such as execution time calculations, but we should also maintain
// the manual part to retain the precision of a skilled developer injecting manually written code for the analysis. It could take the form of an editor, like what we've been using.
//   My idea is for it to be helpful for both new developers and experienced ones, by aiding the newer developers by giving them an overview and general functions they can use "out of the box" and 
// giving the experienced developers a tool which cuts out unnecessary busy work for them.
// 
// Commit your thoughts, we will discuss further when we meet.
