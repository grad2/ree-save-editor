use std::collections::HashMap;
use std::cell::RefCell;

use ree_lib::context::EngineContext;
use ree_lib::rsz::Value;
use ree_lib::types::Guid;

use crate::save::remap::{Remap}; 
use ree_lib::data::DataSource;

pub struct RemapEvaluator<'a> {
    pub engine: &'a EngineContext<'a>,
    pub remaps: &'a HashMap<String, Remap>,

    // THE CACHE: Stores the results of `@query` lookups.
    // Key: (Query_Name, Current_Value_Hash_or_String) -> Value (The resolved RSZ Object)
    // RefCell allows us to mutate the cache even when `RemapEvaluator` is passed by immutable reference `&self`
    query_cache: RefCell<HashMap<(String, String), Value>>,
}

impl<'a> RemapEvaluator<'a> {
    pub fn new(engine: &'a EngineContext<'a>, remaps: &'a HashMap<String, Remap>) -> Self {
        Self {
            engine,
            remaps,
            query_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn eval_data(&self, data_name: &str, remap: &Remap, current_val: &Value) -> Option<String> {
        let source = remap.data.get(data_name)?;

        match source {
            DataSource::MsgLookup { msg_file, target_query, target_field } => {
                let query_def = remap.queries.get(target_query)?;
                let source_rsz_file = match query_def {
                    DataSource::RszQuery { rsz_file, .. } => rsz_file,
                    _ => return None,
                };

                let row_object = self.resolve_query(target_query, remap, current_val)?;

                let guid_val = self.extract_field(source_rsz_file, &row_object, target_field)?;

                let guid = match guid_val {
                    Value::Guid(g) => Guid(g.0),
                    _ => return None,
                };

                self.engine.get_msg_entry(msg_file, &guid)
            }

            DataSource::RszQuery { rsz_file, array_path, match_field } => {
                let result_obj = self.engine.query_rsz_array(rsz_file, array_path, match_field, current_val)?;
                Some(format!("{:?}", result_obj)) 
            }
        }
    }

    fn resolve_query(&self, query_name: &str, remap: &Remap, current_val: &Value) -> Option<Value> {
        let cache_key = (query_name.to_string(), format!("{:?}", current_val));

        if let Some(cached_val) = self.query_cache.borrow().get(&cache_key) {
            return Some(cached_val.clone());
        }

        let query_source = remap.queries.get(query_name)?;

        let result = match query_source {
            DataSource::RszQuery { rsz_file, array_path, match_field } => {
                self.engine.query_rsz_array(rsz_file, array_path, match_field, current_val).cloned()
            }
            _ => None,
        }?;

        self.query_cache.borrow_mut().insert(cache_key, result.clone());

        Some(result)
    }

    fn extract_field(&self, rsz_file: &str, object: &Value, field_name: &str) -> Option<Value> {
        let obj_id = match object {
            Value::Object(id) => *id as usize,
            _ => return None,
        };

        let rsz = self.engine.assets.get_rsz(rsz_file).ok()?; 

        let instance = rsz.instances.get(obj_id)?;
        let type_info = self.engine.rsz_map.get_by_hash(instance.hash)?;
        let field_idx = type_info.get_field_idx(field_name)?;

        instance.fields.get(field_idx).cloned()
    }
}
