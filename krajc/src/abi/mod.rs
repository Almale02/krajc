pub mod system_interface;
pub mod prelude {
    pub use super::system_interface::*;
    pub use stabby::boxed::Box as RBox;
    pub use stabby::closure as rclosure;
    pub use stabby::dynptr;
    pub use stabby::stabby;
    pub use stabby::string::String as RString;
    pub use stabby::vec::Vec as RVec;
    pub use stabby::Dyn;
}
