use std::error::Error;
use std::fmt::Write;

use documented::DocumentedFields;
use toml_edit::{DocumentMut, Item, Table};

use crate::config::{Config, ListenForegroundEvents, ListenNewProcess};

type Result<T, E = Box<dyn Error + Send + Sync>> = std::result::Result<T, E>;

pub fn append_comments(toml: &str) -> Result<String> {
    let mut new_doc = toml.parse::<DocumentMut>()?;

    comment_sub_struct(new_doc.as_table_mut(), Config::get_field_docs);
    Ok(new_doc.to_string())
}

fn comment_sub_struct(
    t: &mut Table,
    get_field: fn(String) -> Result<&'static str, documented::Error>,
) {
    for (mut field, item) in t.iter_mut() {
        let Ok(doc) = get_field(field.to_string()).map(fold_doc) else {
            continue;
        };

        match item {
            Item::Value(_) => field.leaf_decor_mut().set_prefix((doc + "\n").trim_start()),
            Item::Table(t) => {
                let decor = t.decor_mut();
                let prefix = decor.prefix().unwrap();
                decor.set_prefix(doc + prefix.as_str().unwrap_or_default());

                match field.get() {
                    "listen_foreground_events" => {
                        comment_sub_struct(t, ListenForegroundEvents::get_field_docs);
                    }
                    "listen_new_process" => {
                        comment_sub_struct(t, ListenNewProcess::get_field_docs);
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }
}

fn fold_doc(doc: &str) -> String {
    doc.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .fold(String::default(), |mut s, l| {
            let _ = write!(&mut s, "\n# {l}");
            s
        })
}
