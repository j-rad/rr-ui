//! Form Components
//!
//! Reusable form input components for building complex forms.

pub mod choice_box;
pub mod date_picker;
pub mod flow_j_form;
pub mod multi_text_input;
pub mod number_input;
pub mod reality_form;
pub mod switch;
pub mod text_area;
pub mod text_input;
pub mod vless_form;

pub use flow_j_form::FlowJForm;
pub use reality_form::RealityForm;
pub use vless_form::VlessForm;

pub use choice_box::{ChoiceBox, ChoiceBoxOption};
pub use date_picker::DatePicker;
pub use multi_text_input::MultiTextInput;
pub use number_input::NumberInput;
pub use switch::Switch;
pub use text_area::TextArea;
pub use text_input::TextInput;
pub mod db_mimic_form;
pub use db_mimic_form::DbMimicForm;
pub mod slipstream_form;
pub use slipstream_form::SlipstreamForm;
