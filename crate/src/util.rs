


/*
 * Time module contains utilty functions to parse time string values
 */
mod Time {

    extern crate regex;
    use self::regex::Regex;

    #[derive(Debug)]
    struct Iso8601 {
        // Date
        year:   f32,
        month:  f32,
        week:   f32,
        day:    f32,

        // Time
        hour:   f32,
        minute: f32,
        second: f32,
    }

    impl Iso8601 {
        fn to_seconds(&self) -> f32 {
            let mut total: f32 = 0f32;
            total = self.year * 31536000f32;
            total = total + self.month * 2592000f32;
            total = total + self.week * 604800f32;
            total = total + self.day * 86400f32;

            total = total + self.hour * 3600f32;
            total = total + self.minute * 60f32;
            total = total + self.second;

            return total;
        }
    }


        

    // Parse a ISO8601 duration value and returnthe parsed value in seconds
    pub fn iso8601parser(time: &str) -> f32 {
        let re = Regex::new(
            r#"P([\d]*Y)*([\d]*M)*([\d]*W)*([\d]*D)*T([\d]*H)*([\d]*M)*([\d*\.?\d]*S)*"#
        ).unwrap();

        let mut iso8601_time = Iso8601 {
            year:   0f32,
            month:  0f32,
            week:   0f32,
            day:    0f32,
            hour:   0f32,
            minute: 0f32,
            second: 0f32
        };

        for cap in re.captures_iter(time) {
            iso8601_time.year = cap.get(1).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.month = cap.get(2).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.week = cap.get(3).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.day = cap.get(4).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.hour = cap.get(5).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.minute = cap.get(6).map_or(0f32, |m| parse_number(m.as_str()));
            iso8601_time.second = cap.get(7).map_or(0f32, |m| parse_number(m.as_str()));
        }
        return iso8601_time.to_seconds();
    }

    fn parse_number(input: &str) -> f32 {
        let re = Regex::new(
            r#"[\d*\.?\d]*"#
        ).unwrap();
        let default_val = 0f32;
        for cap in re.captures_iter(input) {
            let temp = cap.get(0).map_or("0", |m| m.as_str());
            return temp.parse::<f32>().expect("FAIL");
        }
        default_val
    }
}
/*
 * Dash module contains utilty functions to parse Dash specific tags
 */
pub mod Dash {
    use super::Time::iso8601parser;

    pub fn requested_position_is_valid<'a>(mpd_node: &roxmltree::Node<'a, 'a>, position: f32) -> bool {
        log!(" MPD {:#?}", mpd_node);
        let asset_duration = match extract_attribute("mediaPresentationDuration", &mpd_node) {
            Some(duration) => iso8601parser(duration),
            None => 0f32,
        };
        return position < asset_duration;
    }

    /// Returns the xml Period node for the give n position in playback
    /// 
    /// # Arguments
    /// 
    /// * `node` - A roxmltree node. This is the root node to the document 
    /// * `position` - A u32 value of the position in playback to look into the mpd
    /// 
    /// NOTE: The dash spec specifies that Periods can optionally have a `start` or 
    /// must (if it is multi-period) have a `dutation` attribute. find_period looks 
    /// first for the `start` and if that doesn't exist then we look for a `duration`
    /// attribute
    pub fn find_period<'a, 'b>(node: &'a roxmltree::Node<'b, 'b>, position: f32) -> roxmltree::Node<'b, 'b> {
        let mut previous_period: roxmltree::Node = node.children().find(|n| n.tag_name().name() == "Period").unwrap();
        let mut start = 0f32;
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == "Period" {
                let next_start_option = get_start(&child);
                let mut next_start = 0f32;
                if next_start_option == None {
                    next_start = start;
                    let duration = get_duration(&child).map_or(0f32, |d| d);
                    next_start = next_start + duration;
                } else {
                    next_start = next_start_option.unwrap();
                }
                
                previous_period = child;
                if next_start >= position {
                    return previous_period;
                }
                start = next_start;
            }
        }

        return previous_period;
    }
    
    pub fn find_adaptation_set<'a, 'b>(period_node: &'a roxmltree::Node<'b, 'b>, mime_type: &str, role: &str) -> Option<roxmltree::Node<'b, 'b>> {

        for child in period_node.children() {
            if child.is_element() && child.tag_name().name() == "AdaptationSet" {
                // Make sure we have the correct AdaptationSet by the mimeType and the role
                let found_mime_type = extract_attribute("mimeType", &child).map_or("", |v| v);
                let found_role = find_role_value(&child).map_or("main", |r| r);
                if role == found_role && mime_type == found_mime_type {
                    return Some(child);
                }
            }
        }
        return None;
    }

    /// Returns the media url for the given position and bandwidth
    /// 
    /// # Arguments
    /// 
    /// * `aset_node` - A roxmltree node. This is the AdaptationSet node that contains 
    /// * `bandwidth` - A u32 value of the selected bandwidth determining which Representation to use
    /// * `position` - A f32 value of the position in playback to look into the mpd
    /// 
    /// NOTE: This current implementation only supports SegmentTemplate (excluding SegmentTimeline) of
    /// the Dash spec. get_media_from_adaptation_set is where to decide which segment type (SegmentList,
    /// SegmentBase, SegmentTemplate)
    pub fn get_media_from_adaptation_set<'a, 'b>(aset_node: &'a roxmltree::Node<'b, 'b>, bandwidth: u32, position: f32) -> Option<String> {
        let segment_template = find_child("SegmentTemplate", &aset_node);
        if segment_template == None {
            error!("SegmentTemplate could not be found");
            return None;
        }
        let segment_template = segment_template.unwrap();
        let number = find_segment_index(&segment_template, position);
        if number == None {
            error!("Couldn't find segment index for position {}", bandwidth);
            return None;
        }
        let number = number.unwrap();
        let representation_id = get_representation_id_from_bandwidth(&aset_node, bandwidth);
        if representation_id == None {
            error!("Couldn't find RepresentationID for bandwidth {}", bandwidth);
            return None;
        }
        let representation_id = representation_id.unwrap();
        let media_str_template = extract_attribute("media", &segment_template).unwrap();
        // let ben: &mut String;
        let ben = media_str_template
                    .replace("$RepresentationID$", representation_id)
                    .replace("$Bandwidth$", &bandwidth.to_string())
                    .replace("$Number$", &number.to_string());
        return Some(ben);
    }

    /*
     * find_role_value looks for a `Role` tag  within the adaptation set. If no `Role` is found
     * we default to the value "main" because there should be no other AdaptationSet of the same
     * mime type
     */
    fn find_role_value<'a>(aset_node: &roxmltree::Node<'a, 'a>) -> Option<&'a str> {
        return match find_child("Role", &aset_node) {
            Some(node) => extract_attribute("value", &node),
            None => None,
        }
    }

    fn get_duration(node: &roxmltree::Node) -> Option<f32> {
        return match extract_attribute("duration", &node) {
            Some(val) => Some(iso8601parser(val)),
            None => None,
        }
    }

    fn get_start(node: &roxmltree::Node) -> Option<f32> {
        return match extract_attribute("start", &node) {
            Some(val) => Some(iso8601parser(val)),
            None => None,
        }
    }

    fn extract_attribute<'a>(attribute: &str, node: &roxmltree::Node<'a, 'a>) -> Option<&'a str> {
        for attr in node.attributes() {
            if attr.name() == attribute {
                return Some(attr.value());
            }
        }
        return None;
    }

    fn find_child<'a, 'b>(node_name: &str, node: &'a roxmltree::Node<'b, 'b>) -> Option<roxmltree::Node<'b, 'b>> {
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == node_name {
                return Some(child);
            }
        }
        return None;
    }

    fn find_segment_index<'a>(segment_template_node: & roxmltree::Node<'a, 'a>, position: f32) -> Option<u32> {
        let start_number = match extract_attribute("startNumber", &segment_template_node) {
            Some(start) => start.parse::<u32>().expect("Fail parsing start number"),
            None => 0
        };
        let timescale = match extract_attribute("timescale", &segment_template_node) {
            Some(ts) => ts.parse::<f32>().expect("Fail parsing timescale"),
            None => 1f32
        };
        let segment_duration = match extract_attribute("duration", &segment_template_node) {
            Some(duration) => Some(duration.parse::<f32>().expect("Fail parsing duration")),
            None => None,
        };
        if segment_duration == None {
            error!("No segment duration on SegmentTemplate");
            return None;
        }
        let segment_duration = segment_duration.unwrap();

        let index = (position / (segment_duration / timescale)).floor() as u32 + start_number;
        log!("INDEX: {}", index);
        return Some(index);
    }

    fn get_representation_id_from_bandwidth<'a>(aset_node: & roxmltree::Node<'a, 'a>, bandwidth: u32) -> Option<&'a str> {
        for child in aset_node.children() {
            if child.is_element() && child.tag_name().name() == "Representation" {
                let rep_bandwidth = match extract_attribute("bandwidth", &child) {
                    Some (b) => b.parse::<u32>().expect("Fail parsing bandwidth"),
                    None => 0,
                };
                if rep_bandwidth == bandwidth {
                    return extract_attribute("id", &child);
                }
            }
        }
        return None;
    }
}