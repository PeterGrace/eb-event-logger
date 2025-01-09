use aws_sdk_elasticbeanstalk::types::EventDescription;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use aws_smithy_types_convert::date_time::DateTimeExt;

pub struct MyEventDescription(pub EventDescription);

impl Serialize for MyEventDescription {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("MyEventDescription", 9).unwrap();

        let _ = state.serialize_field("event_date", &self.0.event_date.unwrap().to_chrono_utc().unwrap().to_string().as_str());
        let _ = state.serialize_field("message", &self.0.message());
        let _ = state.serialize_field("application_name", &self.0.application_name());
        let _ = state.serialize_field("version_label", &self.0.version_label());
        let _ = state.serialize_field("template_name", &self.0.template_name());
        let _ = state.serialize_field("environment_name", &self.0.environment_name());
        let _ = state.serialize_field("platform_arn", &self.0.platform_arn());
        let _ = state.serialize_field("request_id",&self.0.request_id());
        let _ = state.serialize_field("severity",&self.0.severity().unwrap().as_str());


        state.end()
    }
}