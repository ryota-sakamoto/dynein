/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License").
 * You may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use ::serde::{Deserialize, Serialize};
use clap::Parser;
use std::error::Error;
use std::ffi::OsString;

/* =================================================
struct / enum / const
================================================= */

const ABOUT_DYNEIN: &str = "\
dynein is a command line tool to interact with DynamoDB tables/data using concise interface.\n\
dynein looks for config files under $HOME/.dynein/ directory.";

#[derive(Parser, Debug)]
#[clap(name = "dynein", about = ABOUT_DYNEIN, version)]
pub struct Dynein {
    #[clap(subcommand)]
    pub child: Option<Sub>,

    /// The region to use (e.g. --region us-east-1). When using DynamodB Local, use `--region local`.
    /// You can use --region option in both top-level and subcommand-level.
    #[clap(short, long, global = true)]
    pub region: Option<String>,

    /// Specify the port number. This option has an effect only when `--region local` is used.
    #[clap(short, long, global = true)]
    pub port: Option<u32>,

    /// Target table of the operation. You can use --table option in both top-level and subcommand-level.
    /// You can store table schema locally by executing `$ dy use`, after that you need not to specify --table on every command.
    #[clap(short, long, global = true)]
    pub table: Option<String>,

    // #[clap(long, required_if_eq("child", "None"), conflicts_with("child"))]
    #[clap(long)]
    pub shell: bool,
}

// NOTE: need to be placed in the same module as Dynein struct
pub fn initialize_from_args() -> Dynein {
    Dynein::parse()
}

pub fn parse_args<I, S>(_input: I) -> Result<Sub, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    Ok(Sub::parse())
    // Sub::clap()
    //     .global_settings(&[
    //         AppSettings::NoBinaryName,
    //         AppSettings::VersionlessSubcommands,
    //     ])
    //     .get_matches_from_safe(input)
    //     .map(|arg| Sub::from_clap(&arg))
    //     .map_err(|e| Box::new(e) as Box<dyn Error>)
}

// structopt derive supports enum(subcommands), or struct (single commands).
// structopt support clap methods e.g. required_if/conflicts_with https://docs.rs/clap/2.32.0/clap/struct.Arg.html
#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum Sub {
    /* =================================================
    Control Plane commands
    ================================================= */
    /// <sub> Admin operations such as creating/updating table or GSI
    #[clap()]
    Admin {
        #[clap(subcommand)]
        grandchild: AdminSub,
    },

    // NOTE: this command is defined both in top-level and sub-subcommand of table family.
    /// List tables in the region. [API: ListTables]
    #[clap(aliases = &["ls"])]
    List {
        /// List DynamoDB tables in all available regions
        #[clap(long)]
        all_regions: bool,
    },

    // NOTE: this command is defined both in top-level and sub-subcommand of table family.
    /// Show detailed information of a table. [API: DescribeTable]
    #[clap(aliases = &["show", "describe", "info"])]
    Desc {
        /// Target table name. Optionally you may specify the target table by --table (-t) option.
        target_table_to_desc: Option<String>,

        /// Show details of all tables in the region
        #[clap(long)]
        all_tables: bool,

        /// Switch output format.
        #[clap(short, long, value_parser = ["yaml" /*, "raw" */ ])]
        output: Option<String>,
    },

    /* =================================================
    Data Plane commands
    ================================================= */
    /// Retrieve items in a table without any condition. [API: Scan]
    #[clap(aliases = &["s"])]
    Scan {
        /// Limit number of items to return.
        #[clap(short, long, default_value = "100")]
        limit: i64,

        /// Attributes to show, separated by commas, which is mapped to ProjectionExpression (e.g. --attributes name,address,age).
        /// Note that primary key(s) are always included in results regardless of what you've passed to --attributes.
        #[clap(short, long)]
        attributes: Option<String>,

        /// Strong consistent read - to make sure retrieve the most up-to-date data. By default (false), eventual consistent reads would occur.
        /// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.ReadConsistency.html
        #[clap(long)]
        consistent_read: bool,

        /// Show only Primary Key(s).
        #[clap(long)]
        keys_only: bool,

        /// Read data from index instead of base table.
        #[clap(short, long)]
        index: Option<String>,

        /// Switch output format.
        #[clap(short, long, value_parser = ["table", "json", "raw"])]
        output: Option<String>,
    },

    /// Retrieve an item by specifying primary key(s). [API: GetItem]
    #[clap(aliases = &["g"])]
    Get {
        /// Partition Key of the target item.
        pval: String,
        /// Sort Key of the target item (if any).
        sval: Option<String>,

        /// Strong consistent read - to make sure retrieve the most up-to-date data. By default (false), eventual consistent reads would occur.
        /// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.ReadConsistency.html
        #[clap(long)]
        consistent_read: bool,

        /// Switch output format.
        #[clap(short, long, value_parser = ["json", "yaml", "raw"])]
        output: Option<String>,
    },

    /// Retrieve items that match conditions. Partition key is required. [API: Query]
    #[clap(aliases = &["q"])]
    Query {
        /// Target Partition Key.
        pval: String,

        /// Additional Sort Key condition which will be converted to KeyConditionExpression.
        /// Valid syntax: ['= 12', '> 12', '>= 12', '< 12', '<= 12', 'between 10 and 99', 'begins_with myVal"]
        #[clap(short, long = "sort-key")]
        sort_key_expression: Option<String>,

        /// Strong consistent read - to make sure retrieve the most up-to-date data. By default (false), eventual consistent reads would occur.
        /// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.ReadConsistency.html
        #[clap(long)]
        consistent_read: bool,

        /// Read data from index instead of base table.
        #[clap(short, long)]
        index: Option<String>,

        /// Limit the number of items to return. By default, the number of items is determined by DynamoDB.
        #[clap(short, long)]
        limit: Option<i64>,

        /// Attributes to show, separated by commas, which is mapped to ProjectionExpression (e.g. --attributes name,address,age).
        /// Note that primary key(s) are always included in results regardless of what you've passed to --attributes.
        #[clap(short, long)]
        attributes: Option<String>,

        /// Show only Primary Key(s).
        #[clap(long)]
        keys_only: bool,

        /// Results of query are always sorted by the sort key value. By default, the sort order is ascending.
        /// Specify --descending to traverse descending order.
        #[clap(short, long)]
        descending: bool,

        /// Switch output format.
        #[clap(short, long, value_parser = ["table", "json", "raw"])]
        output: Option<String>,
    },

    /// Create a new item, or replace an existing item. [API: PutItem]
    #[clap(aliases = &["p"])]
    Put {
        /// Partition Key of the target item.
        pval: String,
        /// Sort Key of the target item (if any).
        sval: Option<String>,

        /// Additional attributes put into the item, which should be valid JSON.
        /// e.g. --item '{"name": "John", "age": 18, "like": ["Apple", "Banana"]}'
        #[clap(short, long)]
        item: Option<String>,
    },

    /// Delete an existing item. [API: DeleteItem]
    #[clap(aliases = &["d", "delete"])]
    Del {
        /// Partition Key of the target item.
        pval: String,
        /// Sort Key of the target item (if any).
        sval: Option<String>,
    },

    /// Update an existing item. [API: UpdateItem]
    ///
    /// This command accepts --set or --remove option and generates DynamoDB's UpdateExpression that is passed to UpdateItem API.
    /// Note that modifying primary key(s) means item replacement in DynamoDB, so updating pk/sk is not allowed in API level.
    /// For more information:
    /// https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_UpdateItem.html
    /// https://docs.amazonaws.cn/en_us/amazondynamodb/latest/developerguide/Expressions.UpdateExpressions.html
    #[clap(aliases = &["update", "u"])]
    Upd {
        /// Partition Key of the target item.
        pval: String,
        /// Sort Key of the target item (if any).
        sval: Option<String>,

        // #[clap(short = "e", long = "expression")] // or, it should be positional option as required?
        // update_expression: String,
        /// SET action to modify or add attribute(s) of an item. --set cannot be used with --remove.
        /// e.g. --set 'name = Alice', --set 'Price = Price + 100', or --set 'Replies = 2, Closed = true, LastUpdated = "2020-02-22T18:10:57Z"'
        #[clap(long, conflicts_with("remove"))]
        set: Option<String>,

        /// REMOVE action to remove attribute(s) from an item. --remove cannot be used with --set.
        /// e.g. --remove 'Category, Rank'
        #[clap(long, conflicts_with("set"))]
        remove: Option<String>,

        // TODO: ConditionExpression support --condition/-c
        /// Increment a Number attribute by 1. e.g. `dy update <keys> --atomic-counter sitePv`.
        #[clap(long)]
        atomic_counter: Option<String>,
    },

    /// Put or Delete multiple items at one time, up to 25 requests. [API: BatchWriteItem]
    ///
    /// https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_BatchWriteItem.html
    #[clap(aliases = &["batch-write-item", "batch-write", "bw"])]
    Bwrite {
        /// Input JSON file path. This input file should be BatchWriteItem input JSON syntax. For more info:
        /// https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_BatchWriteItem.html
        #[clap(long, short)]
        input: String,
    },

    /* =================================================
    Dynein utility commands
    ================================================= */
    /// Switch target table context. After you use the command you don't need to specify table every time, but you may overwrite the target table with --table (-t) option.
    ///
    /// When you execute `use`, dynein retrieves table schema info via DescribeTable API
    /// and stores it in ~/.dynein/ directory.
    #[clap()]
    Use {
        /// Target table name to use. Optionally you may specify the target table by --table (-t) option.
        target_table_to_use: Option<String>,
    },

    /// <sub> Manage configuration files (config.yml and cache.yml) from command line
    #[clap()]
    Config {
        #[clap(subcommand)]
        grandchild: ConfigSub,
    },

    /// Create sample tables and load test data for bootstrapping
    #[clap()]
    Bootstrap {
        #[clap(short, long, conflicts_with("sample"))]
        list: bool,

        #[clap(short, long, conflicts_with("list"))]
        sample: Option<String>,
    },

    /// Export items from a DynamoDB table and save them as CSV/JSON file.
    ///
    /// If you want to achieve best performance, recommendated way is to switch the table to OnDemand mode before export. (e.g. dy admin update table your_table --mode ondemand).{n}
    /// When you export items as JSON (including jsonl, json-compact), all attributes in all items will be exported.{n}
    /// When you export items as CSV, on the other hand, dynein has to know which attributes are to be exported as CSV format requires "column" - i.e. N th column should contain attribute ABC throughout a csv file.
    #[clap()]
    Export {
        /// Output target filename where dynein exports data into.
        #[clap(short, long)]
        output_file: String,

        /// Data format for export items.{n}
        ///   json = JSON format with newline/indent.{n}
        ///   jsonl = JSON Lines (http://jsonlines.org). i.e. one item per line.{n}
        ///   json-compact = JSON format, all items are packed in oneline.{n}
        ///   csv = comma-separated values with header. Use it with --keys-only or --attributes. If neither of them are given dynein will ask you target attributes interactively.
        #[clap(short, long, value_parser = ["csv", "json", "jsonl", "json-compact"])]
        format: Option<String>,

        /// [csv] Specify attributes to export, separated by commas (e.g. --attributes name,address,age). Effective only when --format is 'csv'.{n}
        /// Note that primary key(s) are always included in results regardless of what you've passed to --attributes.
        #[clap(short, long, conflicts_with("keys_only"))]
        attributes: Option<String>,

        /// [csv] Export only Primary Key(s). Effective only when --format is 'csv'.
        #[clap(long, conflicts_with("attributes"))]
        keys_only: bool,
    },

    /// Import items into a DynamoDB table from CSV/JSON file.
    ///
    /// If you want to achieve best performance, recommendated way is to switch the table to OnDemand mode before import. (e.g. dy admin update table your_table --mode ondemand).{n}
    /// When you import items from a CSV file, header names are used to attributes for items. The first one or two column(s) would be primary key(s).
    #[clap()]
    Import {
        /// Filename contains DynamoDB items data. Specify appropriate format with --format option.
        #[clap(short, long)]
        input_file: String,

        /// Data format for import items.{n}
        ///   json = JSON format with newline/indent.{n}
        ///   jsonl = JSON Lines (http://jsonlines.org). i.e. one item per line.{n}
        ///   json-compact = JSON format, all items are packed in oneline.{n}
        ///   csv = comma-separated values with header. Header columns are considered to be DynamoDB attributes.
        #[clap(short, long, value_parser = ["csv", "json", "jsonl", "json-compact"])]
        format: Option<String>,
    },

    /// Take backup of a DynamoDB table using on-demand backup
    ///
    /// For more details: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/BackupRestore.html
    #[clap()]
    Backup {
        /// List existing DynamoDB backups
        #[clap(short, long /*, required_if("all_tables", "true") */)]
        list: bool,

        /// List backups for all tables in the region
        #[clap(long)]
        all_tables: bool,
    },

    /// Restore a DynamoDB table from backup data
    ///
    /// For more details: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/BackupRestore.html
    #[clap()]
    Restore {
        /// Specify backup file. If not specified you can select it interactively.
        #[clap(short, long)]
        backup_name: Option<String>,

        /// Name of the newly restored table. If not specified, default naming rule "<source-table-name>-restore-<timestamp>" would be used.
        #[clap(long)]
        restore_name: Option<String>,
    },
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum AdminSub {
    /// List tables in the region. [API: ListTables]
    #[clap(aliases = &["ls"])]
    List {
        /// List DynamoDB tables in all available regions
        #[clap(long)]
        all_regions: bool,
    },

    /// Show detailed information of a table. [API: DescribeTable]
    #[clap(aliases = &["show", "describe", "info"])]
    Desc {
        /// Target table name. Optionally you may specify the target table by --table (-t) option.
        target_table_to_desc: Option<String>,

        /// Show details of all tables in the region
        #[clap(long)]
        all_tables: bool,

        /// Switch output format.
        #[clap(short, long, value_parser = ["yaml" /*, "raw" */ ])]
        output: Option<String>,
    },

    /// Create new DynamoDB table or GSI. [API: CreateTable, UpdateTable]
    #[clap()]
    Create {
        #[clap(subcommand)]
        target_type: CreateSub,
    },

    /// Update a DynamoDB table. [API: UpdateTable etc]
    #[clap()]
    Update {
        #[clap(subcommand)]
        target_type: UpdateSub,
    },

    /// Delete a DynamoDB table or GSI. [API: DeleteTable]
    #[clap()]
    Delete {
        #[clap(subcommand)]
        target_type: DeleteSub,
    },
    /*
    /// Compare the desired and current state of a DynamoDB table.
    #[clap()]
    Plan {
        /// target table name to create/update.
        name: String,
    },

    /// Create or update DynamoDB tables based on CloudFormation template files (.cfn.yml).
    #[clap()]
    Apply {
    },

    /// Delete all items in the target table.
    #[clap()]
    Truncate {
        /// table name to truncate
        name: String,

        /// Skip interactive confirmation before deleting items.
        #[clap(long)]
        yes: bool,
    },
    */
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum CreateSub {
    /// Create new DynamoDB table with given primary key(s). [API: CreateTable]
    #[clap()]
    Table {
        /// table name to create
        new_table_name: String,

        /// (requried) Primary key(s) of the table. Key name followed by comma and data type (S/N/B).
        /// e.g. for Partition key only table: `--keys myPk,S`, and for Partition and Sort key table `--keys myPk,S mySk,N`
        #[clap(short, long, required = true)]
        keys: Vec<String>,
    },

    /// Create new GSI (global secondary index) for a table with given primary key(s). [API: UpdateTable]
    #[clap()]
    Index {
        /// index name to create
        index_name: String,

        /// (requried) Primary key(s) of the index. Key name followed by comma and data type (S/N/B).
        /// e.g. for Partition key only table: `--keys myPk,S`, and for Partition and Sort key table `--keys myPk,S mySk,N`
        #[clap(short, long, required = true)]
        keys: Vec<String>,
    },
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum UpdateSub {
    /// Update a DynamoDB table.
    #[clap()]
    Table {
        /// table name to update
        table_name_to_update: String,

        /// DynamoDB capacity mode. Availablle values: [provisioned, ondemand].
        /// When you switch from OnDemand to Provisioned mode, you can pass WCU and RCU as well (NOTE: default capacity unit for Provisioned mode is 5).
        #[clap(short, long, value_parser = ["provisioned", "ondemand"])]
        mode: Option<String>,

        /// WCU (write capacity units) for the table. Acceptable only on Provisioned mode.
        #[clap(long)]
        wcu: Option<i64>,

        /// RCU (read capacity units) for the table. Acceptable only on Provisioned mode.
        #[clap(long)]
        rcu: Option<i64>,
        // TODO: support following parameters
        // - sse_enabled: bool, (default false) ... UpdateTable API
        // - stream_enabled: bool, (default false) ... UpdateTable API
        // - ttl_enabled: bool, UpdateTimeToLive API
        // - pitr_enabled: bool, UpdateContinuousBackups API (PITR)
    },
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum DeleteSub {
    /// Delete a DynamoDB table.
    #[clap()]
    Table {
        /// table name to delete
        table_name_to_delete: String,

        /// Skip interactive confirmation before deleting a table.
        #[clap(long)]
        yes: bool,
    },
    // #[clap()]
    // Index {
    // }
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub enum ConfigSub {
    /// Show all configuration in config (config.yml) and cache (cache.yml) files.
    #[clap(aliases = &["show", "current-context"])]
    // for now, as config content is not so large, showing current context == dump all config.
    Dump,

    /// Reset all dynein configuration in the `~/.dynein/` directory. This command initializes dynein related files only and won't remove your data stored in DynamoDB tables.
    #[clap()]
    Clear,
}
