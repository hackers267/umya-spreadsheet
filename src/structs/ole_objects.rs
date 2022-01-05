// oleObjects
use super::OleObject;
use writer::driver::*;
use quick_xml::Reader;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;
use std::io::Cursor;

#[derive(Default, Debug)]
pub struct OleObjects {
    ole_object: Vec<OleObject>,
}
impl OleObjects {
    pub fn get_ole_object(&self)-> &Vec<OleObject> {
        &self.ole_object
    }

    pub fn get_ole_object_mut(&mut self)-> &mut Vec<OleObject> {
        &mut self.ole_object
    }

    pub fn set_ole_object(&mut self, value:OleObject)-> &mut Self {
        self.ole_object.push(value);
        self
    }

    pub(crate) fn set_attributes<R: std::io::BufRead, A: std::io::Read + std::io::Seek>(
        &mut self,
        reader:&mut Reader<R>,
        _e:&BytesStart,
        arv: &mut zip::read::ZipArchive<A>,
        target: &str,
    ) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        b"mc:AlternateContent" => {
                            let mut obj = OleObject::default();
                            obj.set_attributes(reader, e, arv, target);
                            self.set_ole_object(obj);
                        },
                        _ => (),
                    }
                },
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        b"oleObjects" => return,
                        _ => (),
                    }
                },
                Ok(Event::Eof) => panic!("Error not find {} end element", "oleObjects"),
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
    }

    pub(crate) fn write_to(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        r_id: &usize,
    ) {
        if self.ole_object.len() > 0 {
            // oleObjects
            write_start_tag(writer, "oleObjects", vec![], false);

            // mc:AlternateContent
            let mut r = r_id.clone();
            for obj in &self.ole_object {
                obj.write_to(writer, &r);
                r += 2;
            }

            write_end_tag(writer, "oleObjects");
        }
    }
}
