use chrono::format;
use html_parser::{Dom, Element, Node};
use std::collections::HashMap;
use structs::Color;
use structs::Font;
use structs::RichText;
use structs::TextElement;
use structs::UnderlineValues;
use structs::VerticalAlignmentRunValues;

/// Generate rich text from html.
/// # Arguments
/// * `html` - HTML String.
/// # Return value
/// * `Result<RichText, html_parser::Error>`
/// # Examples
/// ```
/// let html = r##"<font color="red">test</font><br><font class="test" color="#48D1CC">TE<b>S</b>T<br/>TEST</font>"##;
/// let richtext = umya_spreadsheet::helper::html::html_to_richtext(html).unwrap();
///
/// let mut book = umya_spreadsheet::new_file();
/// let mut sheet = book.get_sheet_by_name_mut("Sheet1").unwrap();
/// sheet.get_cell_mut("A1").set_rich_text(richtext);
/// // Enable line breaks.
/// sheet
///     .get_cell_mut("A1")
///     .get_style_mut()
///     .get_alignment_mut()
///     .set_wrap_text(true);
/// ```
pub fn html_to_richtext(html: &str) -> Result<RichText, html_parser::Error> {
    html_to_richtext_custom(html, &DataAnalysis::default())
}

/// Use here for custom html parsing.
/// # Arguments
/// * `html` - HTML String.
/// * `method` - struct for analysis.
/// # Return value
/// * `Result<RichText, html_parser::Error>`
pub fn html_to_richtext_custom(
    html: &str,
    method: &AnalysisMethod,
) -> Result<RichText, html_parser::Error> {
    let dom = Dom::parse(html)?;
    let data = read_node(&dom.children, &Vec::new());
    let result = make_rich_text(&data, method);
    Ok(result)
}

fn read_node(node_list: &Vec<Node>, parent_element: &Vec<HfdElement>) -> Vec<HtmlFlatData> {
    let mut result: Vec<HtmlFlatData> = Vec::new();

    if node_list.is_empty() {
        return result;
    }

    let mut data = HtmlFlatData::default();
    data.element.extend_from_slice(parent_element);

    for node in node_list {
        match node {
            Node::Text(text) => {
                data.text = format!("{}{}", data.text, text);
            }
            Node::Element(element) => {
                if &element.name == "br" {
                    data.text = format!("{}{}", data.text, "\n");
                    continue;
                }
                if &data.text != "" {
                    result.push(data);
                    data = HtmlFlatData::default();
                    data.element.append(&mut parent_element.clone());
                }

                let mut elm: HfdElement = HfdElement::default();
                elm.name = element.name.to_string();

                elm.attributes = element
                    .attributes
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            value.as_ref().map(|v| v.to_string()).unwrap_or_default(),
                        )
                    })
                    .collect();

                elm.classes = element.classes.clone();
                data.element.push(elm);

                let mut children = read_node(&element.children, &data.element);
                result.append(&mut children);

                data = HtmlFlatData::default();
                data.element.extend_from_slice(parent_element);
            }
            _ => {}
        }
    }
    if &data.text != "" {
        result.push(data);
    }
    result
}

fn make_rich_text(html_flat_data_list: &Vec<HtmlFlatData>, method: &AnalysisMethod) -> RichText {
    let mut result = RichText::default();

    for html_flat_data in html_flat_data_list {
        let mut font_name: Option<String> = method.font_name(html_flat_data);
        let mut size: Option<f64> = method.size(html_flat_data);
        let mut color: Option<String> = method.color(html_flat_data);
        let mut is_bold: bool = method.is_bold(html_flat_data);
        let mut is_italic: bool = method.is_italic(html_flat_data);
        let mut is_underline: bool = method.is_underline(html_flat_data);
        let mut is_superscript: bool = method.is_superscript(html_flat_data);
        let mut is_subscript: bool = method.is_subscript(html_flat_data);
        let mut is_strikethrough: bool = method.is_strikethrough(html_flat_data);

        let mut text_element = TextElement::default();
        let mut font = Font::default();

        if let Some(v) = font_name {
            font.set_name(v);
        }

        if let Some(v) = size {
            font.set_size(v);
        }

        if let Some(v) = color {
            let argb = v;
            let mut clr = Color::default();
            clr.set_argb(argb);
            font.set_color(clr);
        }

        if is_bold {
            font.set_bold(is_bold);
        }
        if is_italic {
            font.set_italic(is_italic);
        }
        if is_underline {
            font.get_font_underline_mut()
                .set_val(UnderlineValues::Single);
        }
        if is_superscript {
            font.get_vertical_text_alignment_mut()
                .set_val(VerticalAlignmentRunValues::Superscript);
        }
        if is_subscript {
            font.get_vertical_text_alignment_mut()
                .set_val(VerticalAlignmentRunValues::Subscript);
        }
        if is_strikethrough {
            font.set_strikethrough(is_strikethrough);
        }

        text_element.set_text(&html_flat_data.text);
        text_element.set_run_properties(font);
        result.add_rich_text_elements(text_element);
    }
    result
}

#[derive(Clone, Default, Debug)]
pub struct HtmlFlatData {
    text: String,
    element: Vec<HfdElement>,
}

#[derive(Clone, Default, Debug)]
pub struct HfdElement {
    name: String,
    attributes: HashMap<String, String>,
    classes: Vec<String>,
}
impl HfdElement {
    pub fn has_name(&self, name: &str) -> bool {
        self.name == name
    }

    pub fn get_by_name_and_attribute(&self, name: &str, attribute: &str) -> Option<String> {
        self.attributes
            .get(attribute)
            .and_then(|v| (self.name == name).then(|| v.to_string()))
    }

    pub fn contains_class(&self, class: &str) -> bool {
        self.classes.contains(&class.to_string())
    }
}

pub trait AnalysisMethod {
    fn is_tag(&self, html_flat_data: &HtmlFlatData, tag: &str) -> bool;
    fn font_name(&self, html_flat_data: &HtmlFlatData) -> Option<String>;
    fn size(&self, html_flat_data: &HtmlFlatData) -> Option<f64>;
    fn color(&self, html_flat_data: &HtmlFlatData) -> Option<String>;
    fn is_bold(&self, html_flat_data: &HtmlFlatData) -> bool;
    fn is_italic(&self, html_flat_data: &HtmlFlatData) -> bool;
    fn is_underline(&self, html_flat_data: &HtmlFlatData) -> bool;
    fn is_superscript(&self, html_flat_data: &HtmlFlatData) -> bool;
    fn is_subscript(&self, html_flat_data: &HtmlFlatData) -> bool;
    fn is_strikethrough(&self, html_flat_data: &HtmlFlatData) -> bool;
}

#[derive(Clone, Default, Debug)]
struct DataAnalysis {}
impl AnalysisMethod for DataAnalysis {
    fn font_name(&self, html_flat_data: &HtmlFlatData) -> Option<String> {
        html_flat_data
            .element
            .iter()
            .find_map(|element| element.get_by_name_and_attribute("font", "face"))
    }

    fn size(&self, html_flat_data: &HtmlFlatData) -> Option<f64> {
        html_flat_data.element.iter().find_map(|element| {
            element
                .get_by_name_and_attribute("font", "size")
                .and_then(|v| v.parse::<f64>().ok())
        })
    }

    fn color(&self, html_flat_data: &HtmlFlatData) -> Option<String> {
        let mut result: Option<String> = None;
        html_flat_data
            .element
            .iter()
            .flat_map(|element| element.get_by_name_and_attribute("font", "color"))
            .find_map(|v| {
                let color = v.trim_start_matches('#').to_uppercase();
                COLOR_MAP
                    .iter()
                    .find_map(|(key, value)| {
                        (*key.to_uppercase() == color).then(|| value.to_uppercase())
                    })
                    .or_else(|| Some(color))
            })
    }

    fn is_tag(&self, html_flat_data: &HtmlFlatData, tag: &str) -> bool {
        html_flat_data
            .element
            .iter()
            .any(|element| element.has_name(tag))
    }

    fn is_bold(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "b") || self.is_tag(html_flat_data, "strong")
    }

    fn is_italic(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "i") || self.is_tag(html_flat_data, "em")
    }

    fn is_underline(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "u") || self.is_tag(html_flat_data, "ins")
    }

    fn is_superscript(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "sup")
    }

    fn is_subscript(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "sub")
    }

    fn is_strikethrough(&self, html_flat_data: &HtmlFlatData) -> bool {
        self.is_tag(html_flat_data, "del")
    }
}

const COLOR_MAP: &[(&str, &str)] = &[
    ("aliceblue", "f0f8ff"),
    ("antiquewhite", "faebd7"),
    ("antiquewhite1", "ffefdb"),
    ("antiquewhite2", "eedfcc"),
    ("antiquewhite3", "cdc0b0"),
    ("antiquewhite4", "8b8378"),
    ("aqua", "00ffff"),
    ("aquamarine1", "7fffd4"),
    ("aquamarine2", "76eec6"),
    ("aquamarine4", "458b74"),
    ("azure1", "f0ffff"),
    ("azure2", "e0eeee"),
    ("azure3", "c1cdcd"),
    ("azure4", "838b8b"),
    ("beige", "f5f5dc"),
    ("bisque1", "ffe4c4"),
    ("bisque2", "eed5b7"),
    ("bisque3", "cdb79e"),
    ("bisque4", "8b7d6b"),
    ("black", "000000"),
    ("blanchedalmond", "ffebcd"),
    ("blue", "0000ff"),
    ("blue1", "0000ff"),
    ("blue2", "0000ee"),
    ("blue4", "00008b"),
    ("blueviolet", "8a2be2"),
    ("brown", "a52a2a"),
    ("brown1", "ff4040"),
    ("brown2", "ee3b3b"),
    ("brown3", "cd3333"),
    ("brown4", "8b2323"),
    ("burlywood", "deb887"),
    ("burlywood1", "ffd39b"),
    ("burlywood2", "eec591"),
    ("burlywood3", "cdaa7d"),
    ("burlywood4", "8b7355"),
    ("cadetblue", "5f9ea0"),
    ("cadetblue1", "98f5ff"),
    ("cadetblue2", "8ee5ee"),
    ("cadetblue3", "7ac5cd"),
    ("cadetblue4", "53868b"),
    ("chartreuse1", "7fff00"),
    ("chartreuse2", "76ee00"),
    ("chartreuse3", "66cd00"),
    ("chartreuse4", "458b00"),
    ("chocolate", "d2691e"),
    ("chocolate1", "ff7f24"),
    ("chocolate2", "ee7621"),
    ("chocolate3", "cd661d"),
    ("coral", "ff7f50"),
    ("coral1", "ff7256"),
    ("coral2", "ee6a50"),
    ("coral3", "cd5b45"),
    ("coral4", "8b3e2f"),
    ("cornflowerblue", "6495ed"),
    ("cornsilk1", "fff8dc"),
    ("cornsilk2", "eee8cd"),
    ("cornsilk3", "cdc8b1"),
    ("cornsilk4", "8b8878"),
    ("cyan1", "00ffff"),
    ("cyan2", "00eeee"),
    ("cyan3", "00cdcd"),
    ("cyan4", "008b8b"),
    ("darkgoldenrod", "b8860b"),
    ("darkgoldenrod1", "ffb90f"),
    ("darkgoldenrod2", "eead0e"),
    ("darkgoldenrod3", "cd950c"),
    ("darkgoldenrod4", "8b6508"),
    ("darkgreen", "006400"),
    ("darkkhaki", "bdb76b"),
    ("darkolivegreen", "556b2f"),
    ("darkolivegreen1", "caff70"),
    ("darkolivegreen2", "bcee68"),
    ("darkolivegreen3", "a2cd5a"),
    ("darkolivegreen4", "6e8b3d"),
    ("darkorange", "ff8c00"),
    ("darkorange1", "ff7f00"),
    ("darkorange2", "ee7600"),
    ("darkorange3", "cd6600"),
    ("darkorange4", "8b4500"),
    ("darkorchid", "9932cc"),
    ("darkorchid1", "bf3eff"),
    ("darkorchid2", "b23aee"),
    ("darkorchid3", "9a32cd"),
    ("darkorchid4", "68228b"),
    ("darksalmon", "e9967a"),
    ("darkseagreen", "8fbc8f"),
    ("darkseagreen1", "c1ffc1"),
    ("darkseagreen2", "b4eeb4"),
    ("darkseagreen3", "9bcd9b"),
    ("darkseagreen4", "698b69"),
    ("darkslateblue", "483d8b"),
    ("darkslategray", "2f4f4f"),
    ("darkslategray1", "97ffff"),
    ("darkslategray2", "8deeee"),
    ("darkslategray3", "79cdcd"),
    ("darkslategray4", "528b8b"),
    ("darkturquoise", "00ced1"),
    ("darkviolet", "9400d3"),
    ("deeppink1", "ff1493"),
    ("deeppink2", "ee1289"),
    ("deeppink3", "cd1076"),
    ("deeppink4", "8b0a50"),
    ("deepskyblue1", "00bfff"),
    ("deepskyblue2", "00b2ee"),
    ("deepskyblue3", "009acd"),
    ("deepskyblue4", "00688b"),
    ("dimgray", "696969"),
    ("dodgerblue1", "1e90ff"),
    ("dodgerblue2", "1c86ee"),
    ("dodgerblue3", "1874cd"),
    ("dodgerblue4", "104e8b"),
    ("firebrick", "b22222"),
    ("firebrick1", "ff3030"),
    ("firebrick2", "ee2c2c"),
    ("firebrick3", "cd2626"),
    ("firebrick4", "8b1a1a"),
    ("floralwhite", "fffaf0"),
    ("forestgreen", "228b22"),
    ("fuchsia", "ff00ff"),
    ("gainsboro", "dcdcdc"),
    ("ghostwhite", "f8f8ff"),
    ("gold1", "ffd700"),
    ("gold2", "eec900"),
    ("gold3", "cdad00"),
    ("gold4", "8b7500"),
    ("goldenrod", "daa520"),
    ("goldenrod1", "ffc125"),
    ("goldenrod2", "eeb422"),
    ("goldenrod3", "cd9b1d"),
    ("goldenrod4", "8b6914"),
    ("gray", "bebebe"),
    ("gray1", "030303"),
    ("gray10", "1a1a1a"),
    ("gray11", "1c1c1c"),
    ("gray12", "1f1f1f"),
    ("gray13", "212121"),
    ("gray14", "242424"),
    ("gray15", "262626"),
    ("gray16", "292929"),
    ("gray17", "2b2b2b"),
    ("gray18", "2e2e2e"),
    ("gray19", "303030"),
    ("gray2", "050505"),
    ("gray20", "333333"),
    ("gray21", "363636"),
    ("gray22", "383838"),
    ("gray23", "3b3b3b"),
    ("gray24", "3d3d3d"),
    ("gray25", "404040"),
    ("gray26", "424242"),
    ("gray27", "454545"),
    ("gray28", "474747"),
    ("gray29", "4a4a4a"),
    ("gray3", "080808"),
    ("gray30", "4d4d4d"),
    ("gray31", "4f4f4f"),
    ("gray32", "525252"),
    ("gray33", "545454"),
    ("gray34", "575757"),
    ("gray35", "595959"),
    ("gray36", "5c5c5c"),
    ("gray37", "5e5e5e"),
    ("gray38", "616161"),
    ("gray39", "636363"),
    ("gray4", "0a0a0a"),
    ("gray40", "666666"),
    ("gray41", "696969"),
    ("gray42", "6b6b6b"),
    ("gray43", "6e6e6e"),
    ("gray44", "707070"),
    ("gray45", "737373"),
    ("gray46", "757575"),
    ("gray47", "787878"),
    ("gray48", "7a7a7a"),
    ("gray49", "7d7d7d"),
    ("gray5", "0d0d0d"),
    ("gray50", "7f7f7f"),
    ("gray51", "828282"),
    ("gray52", "858585"),
    ("gray53", "878787"),
    ("gray54", "8a8a8a"),
    ("gray55", "8c8c8c"),
    ("gray56", "8f8f8f"),
    ("gray57", "919191"),
    ("gray58", "949494"),
    ("gray59", "969696"),
    ("gray6", "0f0f0f"),
    ("gray60", "999999"),
    ("gray61", "9c9c9c"),
    ("gray62", "9e9e9e"),
    ("gray63", "a1a1a1"),
    ("gray64", "a3a3a3"),
    ("gray65", "a6a6a6"),
    ("gray66", "a8a8a8"),
    ("gray67", "ababab"),
    ("gray68", "adadad"),
    ("gray69", "b0b0b0"),
    ("gray7", "121212"),
    ("gray70", "b3b3b3"),
    ("gray71", "b5b5b5"),
    ("gray72", "b8b8b8"),
    ("gray73", "bababa"),
    ("gray74", "bdbdbd"),
    ("gray75", "bfbfbf"),
    ("gray76", "c2c2c2"),
    ("gray77", "c4c4c4"),
    ("gray78", "c7c7c7"),
    ("gray79", "c9c9c9"),
    ("gray8", "141414"),
    ("gray80", "cccccc"),
    ("gray81", "cfcfcf"),
    ("gray82", "d1d1d1"),
    ("gray83", "d4d4d4"),
    ("gray84", "d6d6d6"),
    ("gray85", "d9d9d9"),
    ("gray86", "dbdbdb"),
    ("gray87", "dedede"),
    ("gray88", "e0e0e0"),
    ("gray89", "e3e3e3"),
    ("gray9", "171717"),
    ("gray90", "e5e5e5"),
    ("gray91", "e8e8e8"),
    ("gray92", "ebebeb"),
    ("gray93", "ededed"),
    ("gray94", "f0f0f0"),
    ("gray95", "f2f2f2"),
    ("gray97", "f7f7f7"),
    ("gray98", "fafafa"),
    ("gray99", "fcfcfc"),
    ("green", "00ff00"),
    ("green1", "00ff00"),
    ("green2", "00ee00"),
    ("green3", "00cd00"),
    ("green4", "008b00"),
    ("greenyellow", "adff2f"),
    ("honeydew1", "f0fff0"),
    ("honeydew2", "e0eee0"),
    ("honeydew3", "c1cdc1"),
    ("honeydew4", "838b83"),
    ("hotpink", "ff69b4"),
    ("hotpink1", "ff6eb4"),
    ("hotpink2", "ee6aa7"),
    ("hotpink3", "cd6090"),
    ("hotpink4", "8b3a62"),
    ("indianred", "cd5c5c"),
    ("indianred1", "ff6a6a"),
    ("indianred2", "ee6363"),
    ("indianred3", "cd5555"),
    ("indianred4", "8b3a3a"),
    ("ivory1", "fffff0"),
    ("ivory2", "eeeee0"),
    ("ivory3", "cdcdc1"),
    ("ivory4", "8b8b83"),
    ("khaki", "f0e68c"),
    ("khaki1", "fff68f"),
    ("khaki2", "eee685"),
    ("khaki3", "cdc673"),
    ("khaki4", "8b864e"),
    ("lavender", "e6e6fa"),
    ("lavenderblush1", "fff0f5"),
    ("lavenderblush2", "eee0e5"),
    ("lavenderblush3", "cdc1c5"),
    ("lavenderblush4", "8b8386"),
    ("lawngreen", "7cfc00"),
    ("lemonchiffon1", "fffacd"),
    ("lemonchiffon2", "eee9bf"),
    ("lemonchiffon3", "cdc9a5"),
    ("lemonchiffon4", "8b8970"),
    ("light", "eedd82"),
    ("lightblue", "add8e6"),
    ("lightblue1", "bfefff"),
    ("lightblue2", "b2dfee"),
    ("lightblue3", "9ac0cd"),
    ("lightblue4", "68838b"),
    ("lightcoral", "f08080"),
    ("lightcyan1", "e0ffff"),
    ("lightcyan2", "d1eeee"),
    ("lightcyan3", "b4cdcd"),
    ("lightcyan4", "7a8b8b"),
    ("lightgoldenrod1", "ffec8b"),
    ("lightgoldenrod2", "eedc82"),
    ("lightgoldenrod3", "cdbe70"),
    ("lightgoldenrod4", "8b814c"),
    ("lightgoldenrodyellow", "fafad2"),
    ("lightgray", "d3d3d3"),
    ("lightpink", "ffb6c1"),
    ("lightpink1", "ffaeb9"),
    ("lightpink2", "eea2ad"),
    ("lightpink3", "cd8c95"),
    ("lightpink4", "8b5f65"),
    ("lightsalmon1", "ffa07a"),
    ("lightsalmon2", "ee9572"),
    ("lightsalmon3", "cd8162"),
    ("lightsalmon4", "8b5742"),
    ("lightseagreen", "20b2aa"),
    ("lightskyblue", "87cefa"),
    ("lightskyblue1", "b0e2ff"),
    ("lightskyblue2", "a4d3ee"),
    ("lightskyblue3", "8db6cd"),
    ("lightskyblue4", "607b8b"),
    ("lightslateblue", "8470ff"),
    ("lightslategray", "778899"),
    ("lightsteelblue", "b0c4de"),
    ("lightsteelblue1", "cae1ff"),
    ("lightsteelblue2", "bcd2ee"),
    ("lightsteelblue3", "a2b5cd"),
    ("lightsteelblue4", "6e7b8b"),
    ("lightyellow1", "ffffe0"),
    ("lightyellow2", "eeeed1"),
    ("lightyellow3", "cdcdb4"),
    ("lightyellow4", "8b8b7a"),
    ("lime", "00ff00"),
    ("limegreen", "32cd32"),
    ("linen", "faf0e6"),
    ("magenta", "ff00ff"),
    ("magenta2", "ee00ee"),
    ("magenta3", "cd00cd"),
    ("magenta4", "8b008b"),
    ("maroon", "b03060"),
    ("maroon1", "ff34b3"),
    ("maroon2", "ee30a7"),
    ("maroon3", "cd2990"),
    ("maroon4", "8b1c62"),
    ("medium", "66cdaa"),
    ("mediumaquamarine", "66cdaa"),
    ("mediumblue", "0000cd"),
    ("mediumorchid", "ba55d3"),
    ("mediumorchid1", "e066ff"),
    ("mediumorchid2", "d15fee"),
    ("mediumorchid3", "b452cd"),
    ("mediumorchid4", "7a378b"),
    ("mediumpurple", "9370db"),
    ("mediumpurple1", "ab82ff"),
    ("mediumpurple2", "9f79ee"),
    ("mediumpurple3", "8968cd"),
    ("mediumpurple4", "5d478b"),
    ("mediumseagreen", "3cb371"),
    ("mediumslateblue", "7b68ee"),
    ("mediumspringgreen", "00fa9a"),
    ("mediumturquoise", "48d1cc"),
    ("mediumvioletred", "c71585"),
    ("midnightblue", "191970"),
    ("mintcream", "f5fffa"),
    ("mistyrose1", "ffe4e1"),
    ("mistyrose2", "eed5d2"),
    ("mistyrose3", "cdb7b5"),
    ("mistyrose4", "8b7d7b"),
    ("moccasin", "ffe4b5"),
    ("navajowhite1", "ffdead"),
    ("navajowhite2", "eecfa1"),
    ("navajowhite3", "cdb38b"),
    ("navajowhite4", "8b795e"),
    ("navy", "000080"),
    ("navyblue", "000080"),
    ("oldlace", "fdf5e6"),
    ("olive", "808000"),
    ("olivedrab", "6b8e23"),
    ("olivedrab1", "c0ff3e"),
    ("olivedrab2", "b3ee3a"),
    ("olivedrab4", "698b22"),
    ("orange", "ffa500"),
    ("orange1", "ffa500"),
    ("orange2", "ee9a00"),
    ("orange3", "cd8500"),
    ("orange4", "8b5a00"),
    ("orangered1", "ff4500"),
    ("orangered2", "ee4000"),
    ("orangered3", "cd3700"),
    ("orangered4", "8b2500"),
    ("orchid", "da70d6"),
    ("orchid1", "ff83fa"),
    ("orchid2", "ee7ae9"),
    ("orchid3", "cd69c9"),
    ("orchid4", "8b4789"),
    ("pale", "db7093"),
    ("palegoldenrod", "eee8aa"),
    ("palegreen", "98fb98"),
    ("palegreen1", "9aff9a"),
    ("palegreen2", "90ee90"),
    ("palegreen3", "7ccd7c"),
    ("palegreen4", "548b54"),
    ("paleturquoise", "afeeee"),
    ("paleturquoise1", "bbffff"),
    ("paleturquoise2", "aeeeee"),
    ("paleturquoise3", "96cdcd"),
    ("paleturquoise4", "668b8b"),
    ("palevioletred", "db7093"),
    ("palevioletred1", "ff82ab"),
    ("palevioletred2", "ee799f"),
    ("palevioletred3", "cd6889"),
    ("palevioletred4", "8b475d"),
    ("papayawhip", "ffefd5"),
    ("peachpuff1", "ffdab9"),
    ("peachpuff2", "eecbad"),
    ("peachpuff3", "cdaf95"),
    ("peachpuff4", "8b7765"),
    ("pink", "ffc0cb"),
    ("pink1", "ffb5c5"),
    ("pink2", "eea9b8"),
    ("pink3", "cd919e"),
    ("pink4", "8b636c"),
    ("plum", "dda0dd"),
    ("plum1", "ffbbff"),
    ("plum2", "eeaeee"),
    ("plum3", "cd96cd"),
    ("plum4", "8b668b"),
    ("powderblue", "b0e0e6"),
    ("purple", "a020f0"),
    ("rebeccapurple", "663399"),
    ("purple1", "9b30ff"),
    ("purple2", "912cee"),
    ("purple3", "7d26cd"),
    ("purple4", "551a8b"),
    ("red", "ff0000"),
    ("red1", "ff0000"),
    ("red2", "ee0000"),
    ("red3", "cd0000"),
    ("red4", "8b0000"),
    ("rosybrown", "bc8f8f"),
    ("rosybrown1", "ffc1c1"),
    ("rosybrown2", "eeb4b4"),
    ("rosybrown3", "cd9b9b"),
    ("rosybrown4", "8b6969"),
    ("royalblue", "4169e1"),
    ("royalblue1", "4876ff"),
    ("royalblue2", "436eee"),
    ("royalblue3", "3a5fcd"),
    ("royalblue4", "27408b"),
    ("saddlebrown", "8b4513"),
    ("salmon", "fa8072"),
    ("salmon1", "ff8c69"),
    ("salmon2", "ee8262"),
    ("salmon3", "cd7054"),
    ("salmon4", "8b4c39"),
    ("sandybrown", "f4a460"),
    ("seagreen1", "54ff9f"),
    ("seagreen2", "4eee94"),
    ("seagreen3", "43cd80"),
    ("seagreen4", "2e8b57"),
    ("seashell1", "fff5ee"),
    ("seashell2", "eee5de"),
    ("seashell3", "cdc5bf"),
    ("seashell4", "8b8682"),
    ("sienna", "a0522d"),
    ("sienna1", "ff8247"),
    ("sienna2", "ee7942"),
    ("sienna3", "cd6839"),
    ("sienna4", "8b4726"),
    ("silver", "c0c0c0"),
    ("skyblue", "87ceeb"),
    ("skyblue1", "87ceff"),
    ("skyblue2", "7ec0ee"),
    ("skyblue3", "6ca6cd"),
    ("skyblue4", "4a708b"),
    ("slateblue", "6a5acd"),
    ("slateblue1", "836fff"),
    ("slateblue2", "7a67ee"),
    ("slateblue3", "6959cd"),
    ("slateblue4", "473c8b"),
    ("slategray", "708090"),
    ("slategray1", "c6e2ff"),
    ("slategray2", "b9d3ee"),
    ("slategray3", "9fb6cd"),
    ("slategray4", "6c7b8b"),
    ("snow1", "fffafa"),
    ("snow2", "eee9e9"),
    ("snow3", "cdc9c9"),
    ("snow4", "8b8989"),
    ("springgreen1", "00ff7f"),
    ("springgreen2", "00ee76"),
    ("springgreen3", "00cd66"),
    ("springgreen4", "008b45"),
    ("steelblue", "4682b4"),
    ("steelblue1", "63b8ff"),
    ("steelblue2", "5cacee"),
    ("steelblue3", "4f94cd"),
    ("steelblue4", "36648b"),
    ("tan", "d2b48c"),
    ("tan1", "ffa54f"),
    ("tan2", "ee9a49"),
    ("tan3", "cd853f"),
    ("tan4", "8b5a2b"),
    ("teal", "008080"),
    ("thistle", "d8bfd8"),
    ("thistle1", "ffe1ff"),
    ("thistle2", "eed2ee"),
    ("thistle3", "cdb5cd"),
    ("thistle4", "8b7b8b"),
    ("tomato1", "ff6347"),
    ("tomato2", "ee5c42"),
    ("tomato3", "cd4f39"),
    ("tomato4", "8b3626"),
    ("turquoise", "40e0d0"),
    ("turquoise1", "00f5ff"),
    ("turquoise2", "00e5ee"),
    ("turquoise3", "00c5cd"),
    ("turquoise4", "00868b"),
    ("violet", "ee82ee"),
    ("violetred", "d02090"),
    ("violetred1", "ff3e96"),
    ("violetred2", "ee3a8c"),
    ("violetred3", "cd3278"),
    ("violetred4", "8b2252"),
    ("wheat", "f5deb3"),
    ("wheat1", "ffe7ba"),
    ("wheat2", "eed8ae"),
    ("wheat3", "cdba96"),
    ("wheat4", "8b7e66"),
    ("white", "ffffff"),
    ("whitesmoke", "f5f5f5"),
    ("yellow", "ffff00"),
    ("yellow1", "ffff00"),
    ("yellow2", "eeee00"),
    ("yellow3", "cdcd00"),
    ("yellow4", "8b8b00"),
    ("yellowgreen", "9acd32"),
];

#[test]
fn convert_test() {
    let html = r#"<font color="red">test</font><br><font class="test" color="green">TE<b>S</b>T<br/>TEST</font>"#;
    let result = html_to_richtext(html).unwrap();
}