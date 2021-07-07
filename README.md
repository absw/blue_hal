# Blue HAL

Embedded Hardware Abstraction Layer developed at [Bluefruit Software]
(https://www.bluefruit.co.uk/). Implementations are mostly only coupled to 
ARM Cortex-M.

`blue_hal` contains most drivers used in the [Loadstone]
(https://github.com/absw/loadstone) secure bootloader project.

# Connection to the Rust embedded ecosystem

`blue_hal` started primarily as an in-house learning project, so there's some
amount of duplication between the drivers contained in this repository and 
some of the preexisting embedded-hal implementations, specially around the
early stm32 drivers. 

The drivers in `blue_hal` don't directly implement the [embedded-hal]
(https://github.com/rust-embedded/embedded-hal) interfaces. Instead, `blue_hal`
offers its own set of abstractions which made sense during the development of
`Loadstone`. The plan is to support `embedded-hal` directly in the future.

# Structure

* `src/hal` contains all abstract interfaces.
* `src/hal/doubles` is a test only module that contains test doubles 
  (fakes, mocks, etc) for most drivers.
* `src/drivers` contains concrete driver implementations. These are nested
  by specificity, with the MCU family or chip vendor always referenced in
  the module structure.
* `src/utilities` contains general purpose code applicable to multiple drivers.
