use duckdb::{
    vtab::{BindInfo, DataChunk, Free, FunctionInfo, InitInfo, LogicalType, LogicalTypeId, VTab},
    Connection, Result,
};

use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;
use croner::Cron;
use duckdb_loadable_macros::duckdb_entrypoint;
use ffi::duckdb_vector_size;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::{c_char, c_void},
    ptr::null_mut,
};

#[repr(C)]
struct CronBindData {
    // The cron expression.
    pattern: *mut Cron,
    start: DateTime<chrono_tz::Tz>,
    until: DateTime<chrono_tz::Tz>,
    timezone: Tz,
}

impl Free for CronBindData {
    fn free(&mut self) {
        unsafe {
            if self.pattern.is_null() {
                return;
            }
            drop(Box::from_raw(self.pattern));
        }
    }
}

#[repr(C)]
struct CronInitData {
    done: bool,
}

struct CronVTab;

impl Free for CronInitData {}

impl VTab for CronVTab {
    type InitData = CronInitData;
    type BindData = CronBindData;

    unsafe fn bind(
        bind: &BindInfo,
        data: *mut CronBindData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        bind.add_result_column("cron", LogicalType::new(LogicalTypeId::TimestampS));

        let pattern = bind.get_parameter(0).to_string();

        match Cron::new(&pattern)
            .with_seconds_optional()
            .with_dom_and_dow()
            .parse()
        {
            Ok(pattern) => {
                (*data).pattern = Box::into_raw(Box::new(pattern));
            }
            Err(err) => {
                let error = format!("Failed to parse cron expression: {}", err);
                (*data).pattern = null_mut();
                bind.set_error(&error);
            }
        }
        let utc_time: Tz = "UTC".parse().expect("UTC is an expected time zone");

        (*data).timezone = match bind.get_named_parameter("timezone") {
            Some(timezone) => timezone.to_string().parse().unwrap_or_else(|_| {
                bind.set_error("Invalid or unknown time zone");
                utc_time
            }),
            None => utc_time,
        };

        let now: DateTime<Tz> = Local::now().with_timezone(&(*data).timezone);
        let now_utc: DateTime<Utc> = Local::now().into();
        // This isn't getting the proper value, so I'm a big confused.
        (*data).start = match bind.get_named_parameter("start") {
            Some(value) => DateTime::from_timestamp(value.to_int64_timestamp() / 1000000, 0)
                .unwrap_or_else(|| {
                    bind.set_error("Invalid starting time");
                    now_utc
                })
                .with_timezone(&(*data).timezone),
            None => now,
        };

        (*data).until = match bind.get_named_parameter("until") {
            Some(value) => DateTime::from_timestamp(value.to_int64_timestamp() / 1000000, 0)
                .unwrap_or_else(|| {
                    bind.set_error("Invalid until time");
                    now_utc
                })
                .with_timezone(&(*data).timezone),
            None => now,
        };

        Ok(())
    }

    unsafe fn init(
        _: &InitInfo,
        data: *mut CronInitData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            (*data).done = false;
        }
        Ok(())
    }

    unsafe fn func(
        func: &FunctionInfo,
        output: &mut DataChunk,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_info = func.get_init_data::<CronInitData>();
        let bind_info = func.get_bind_data::<CronBindData>();

        unsafe {
            let mut vector = output.flat_vector(0);

            if (*init_info).done {
                output.set_len(0)
            } else {
                // DuckDB has a limit to its vector size, respect it.
                let max_items: usize = duckdb_vector_size().try_into().unwrap();
                let mut item_count: usize = 0;

                let timestamps: Vec<i64> = (*(*bind_info).pattern)
                    .iter_from((*bind_info).start)
                    .take_while(|&x| {
                        if ((*bind_info).start == (*bind_info).until && item_count == 0)
                            || (x <= (*bind_info).until && item_count < max_items)
                        {
                            item_count += 1;
                            (*bind_info).start = x;
                            true
                        } else {
                            false
                        }
                    })
                    .map(|x| x.timestamp())
                    .collect::<Vec<i64>>();

                output.set_len(timestamps.len());

                vector.copy(&timestamps);

                // If the number of timestamps produced is less than the max_items
                // it means that the until limit has been reached.
                (*init_info).done = timestamps.len() < max_items;
            }
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalType>> {
        // This is the single parameter which is the cron pattern.
        Some(vec![LogicalType::new(LogicalTypeId::Varchar)])
    }

    fn named_parameters() -> Option<Vec<(String, LogicalType)>> {
        Some(vec![
            (
                "start".to_string(),
                LogicalType::new(LogicalTypeId::Timestamp),
            ),
            (
                "until".to_string(),
                LogicalType::new(LogicalTypeId::Timestamp),
            ),
            (
                "timezone".to_string(),
                LogicalType::new(LogicalTypeId::Varchar),
            ),
        ])
    }
}

// Exposes a extern C function named "libcrontab_init" in the compiled dynamic library,
// the "entrypoint" that duckdb will use to load the extension.
#[duckdb_entrypoint]
pub fn libcrontab_init(conn: Connection) -> Result<(), Box<dyn Error>> {
    conn.register_table_function::<CronVTab>("cron")?;

    Ok(())
}
