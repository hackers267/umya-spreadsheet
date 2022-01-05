//xdr:cNvPr
use super::super::super::StringValue;
use super::super::super::UInt32Value;
use super::super::super::BooleanValue;
use writer::driver::*;
use reader::driver::*;
use quick_xml::Reader;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;
use std::io::Cursor;

#[derive(Default, Debug)]
pub struct NonVisualDrawingProperties  {
    name: StringValue,
    id: UInt32Value,
    hidden: BooleanValue,
    shape_id: StringValue,
}
impl NonVisualDrawingProperties  {
    pub fn get_name(&self) -> &str {
        &self.name.get_value()
    }

    pub fn set_name<S: Into<String>>(&mut self, value:S) {
        self.name.set_value(value);
    }

    pub fn get_id(&self) -> &u32 {
        &self.id.get_value()
    }

    pub fn set_id(&mut self, value:u32) {
        self.id.set_value(value);
    }

    pub fn get_hidden(&self) -> &bool {
        &self.hidden.get_value()
    }

    pub fn set_hidden(&mut self, value:bool) {
        self.hidden.set_value(value);
    }

    pub fn get_shape_id(&self) -> &str {
        &self.shape_id.get_value()
    }

    pub fn set_shape_id<S: Into<String>>(&mut self, value:S) {
        self.shape_id.set_value(value);
    }

    pub(crate) fn set_attributes<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        e: &BytesStart,
        empty_flg: bool,
    ) {
        &mut self.id.set_value_string(get_attribute(e, b"id").unwrap());
        &mut self.name.set_value_string(get_attribute(e, b"name").unwrap());
        match get_attribute(e, b"hidden") {
            Some(v) => {&mut self.hidden.set_value_string(v);},
            None => {}
        }

        if empty_flg {
            return;
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    match e.name() {
                        b"a14:compatExt" => {
                            &mut self.set_shape_id(get_attribute(e, b"spid").unwrap());
                        },
                        _ => (),
                    }
                },
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        b"a:extLst" => return,
                        _ => (),
                    }
                },
                Ok(Event::Eof) => panic!("Error not find {} end element", "a:extLst"),
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
    }

    pub(crate) fn write_to(&self, writer: &mut Writer<Cursor<Vec<u8>>>) {
        let with_inner = self.shape_id.has_value();
        // xdr:cNvPr
        let mut attributes: Vec<(&str, &str)> = Vec::new();
        attributes.push(("id", &self.id.get_value_string()));
        attributes.push(("name", &self.name.get_value_string()));
        if self.hidden.has_value() {
            attributes.push(("hidden", &self.hidden.get_value_string()));
        }
        write_start_tag(writer, "xdr:cNvPr", attributes, !with_inner);

        if with_inner {
            write_start_tag(writer, "a:extLst", vec![], false);
            write_start_tag(writer, "a:ext", vec![
                (("uri","{63B3BB69-23CF-44E3-9099-C40C66FF867C}")),
            ], false);
            write_start_tag(writer, "a14:compatExt", vec![
                (("spid", &self.shape_id.get_value_string())),
            ], true);

            write_end_tag(writer, "a:ext");
            write_end_tag(writer, "a:extLst");
            write_end_tag(writer, "xdr:cNvPr");
        }
    }
}
