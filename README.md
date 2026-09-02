# rrdb

![Rust](https://img.shields.io/badge/language-Rust-red) ![version 0.0.3 alpha](https://img.shields.io/badge/version-0.0.3%20alpha-brightgreen) [![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/myyrakle/rrdb/blob/master/LICENSE)

**English** | [한국어](README.ko.md)

Rust-based RDB

## not complete

---

### Installation

Use cargo.

```bash
cargo install rrdb
```

- Platform-specific initialization (Linux)

Create a symbolic link and run the initialization.

```bash
sudo ln -s /home/$USER/.cargo/bin/rrdb /usr/bin/rrdb
sudo rrdb init # initialize data directory
sudo rrdb daemon # register daemon
```

- Platform-specific initialization (MacOS)

Create a symbolic link and run the initialization.

```bash
sudo ln -s $HOME/.cargo/bin/rrdb /usr/local/bin/rrdb
sudo rrdb init
sudo rrdb daemon
```

- Platform-specific initialization (Windows)

Run PowerShell as administrator and execute the following commands.

```powershell
mkdir 'C:\Program Files\rrdb'
cp ~/.cargo/bin/rrdb.exe 'C:\Program Files\rrdb\'
'C:\Program Files\rrdb\rrdb.exe' init
```

---

### Basic Usage

#### Server

```bash
# Initialize storage
cargo run --bin rrdb init
# Initialize storage in a specific directory
cargo run --bin rrdb init --base-path local-test
# Register and run the daemon
cargo run --bin rrdb daemon
# Run the server
cargo run --bin rrdb run
# Run the server with a specific directory
cargo run --bin rrdb run --base-path local-test
```

#### Docker

```bash
docker build -t rrdb:local .
docker run --rm -p 22208:22208 -v rrdb-data:/var/lib/rrdb rrdb:local
```

#### Client

```bash
psql -U rrdb -p 22208 --host 0.0.0.0
```

---

### Syntax

1. Keywords are case-insensitive.
2. Strings are delimited by single quotes ('), and a quote inside a string is escaped by doubling it.
3. Identifiers can be plain text, or delimited by double quotes (").

#### Database

```sql
# List databases
SHOW DATABASES;
```

```sql
# Create a database
CREATE DATABASE "database name";
```

```sql
# Drop a database
DROP DATABASE "database name";
```

```sql
# Alter a database
ALTER DATABASE "from name" rename to "to name";
```

```sql
# Change the current database
USE "database name";
or
\c "database name";
```

#### Table

```sql
# List tables
SHOW TABLES
```

```sql
# Show table details
DESC "table name"
```

```sql
# Create a table
# (table_constraint will be supported later.)
CREATE TABLE [ IF NOT EXISTS ] "table name"
(
    [
        {
            "column name" data_type  [ column_constraint [ ... ] ]
        }
        [, ... ]
    ]
)

# column_constraint is one of the following forms.
# (CONSTRAINT, CHECK, UNIQUE, REFERENCES, etc. will be supported later.)
{
    NOT NULL |
    NULL |
    DEFAULT default_expr |
    PRIMARY KEY index_parameters
}
```

```sql
# Alter a table

1. ALTER TABLE [ IF EXISTS ] name
    action
2. ALTER TABLE [ IF EXISTS ] name
    RENAME [ COLUMN ] column_name TO new_column_name
3. ALTER TABLE [ IF EXISTS ] name
    RENAME TO new_name

# action is one of the following:

1. ADD [ COLUMN ] column_name data_type [ column_constraint [ ... ] ] # [IF NOT EXISTS] syntax to be added later
2. DROP [ COLUMN ]  column_name # [ IF EXISTS ] syntax to be added later
3. ALTER [ COLUMN ] column_name [ SET DATA ] TYPE data_type
4. ALTER [ COLUMN ] column_name SET DEFAULT expression
5. ALTER [ COLUMN ] column_name DROP DEFAULT
6. ALTER [ COLUMN ] column_name { SET | DROP } NOT NULL
```

#### Insert

```sql
INSERT INTO table_name ( column_name [, ...] )
{
    VALUES ( { expression | DEFAULT } [, ...] ) [, ...]
    |
    select_query
}
```

#### Select

```sql
SELECT
    [ * | expression [ [ AS ] output_name ] [, ...] ]
[ FROM from_item [, ...] ]
[ WHERE condition ]
[ GROUP BY grouping_element [, ...] ]
[ HAVING condition ]
[ ORDER BY expression [ ASC | DESC ] [ NULLS { FIRST | LAST } ] [, ...] ]
[ LIMIT limit_number ]
[ OFFSET offset_number ]

from_item is one of the following:
1. table_name  [ [ AS ] alias ]
2. ( select ) [ AS ] alias
```

#### Update

```sql
UPDATE table_name
SET { column_name = { expression } } [, ...]
[ WHERE condition ]
```

#### Delete

```sql
DELETE FROM table_name
[ WHERE condition ]
```
