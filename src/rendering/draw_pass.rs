use super::material::MaterialGeneric;

pub struct DrawPass {
    pub mat: Box<dyn MaterialGeneric>,
}
