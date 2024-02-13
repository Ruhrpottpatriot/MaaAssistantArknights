# MAARS - Iron(III) oxide bindings to MAA

MAARs is a Rust wrapper around the [MAA Core
Lib](https://github.com/MaaAssistantArknights/MaaAssistantArknights). The bindings are
desinged to leverage Rust's rich type systems and compiler to make writing applications in
Rust as easy as possible. The main goal for this crate is to provide an _unsafe-free_ and
idiomatic API that delibereately hides the actualy C FFI functions and types from the
developer.


## Features

This create contains wrappers around most types to make them secure and add more features
to them that are interesting to Rust users, e.g. numerical id's, such as being equatable,
comparable and cloneable, which the original FFI type might not be. This crate differs
from the maa-sys crate found in the maa-cli project. That crate still exposes some FFI
types to the user, which this crate deliberately avoids. Furthermore, there are some bugs
and incorrect choices in the library, most notably around paths on windows.


## Roadmap
* [ ] **Low Level Wrapper**: While the current wrapper tries its best with performance, it's
  not the main priority. A lower level wrapper would eschew some features in favor for a
  more direct access to the FFI
* [ ] **Libloading**: The maa-sys crate in the
  [maa-cli](https://github.com/MaaAssistantArknights/maa-cli) repository uses libloading
  to load the MAA library. Libloading would allow a dynamic loading of the core MAA
  libraryf without having to know the exact path of the library files. This might be a
  suitable approach compared to the manual linking of this library against the C/C++ MAA
  library.

