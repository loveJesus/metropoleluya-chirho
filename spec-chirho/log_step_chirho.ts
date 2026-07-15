#!/usr/bin/env bun
// For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV)
// Logs one row per unit of work into spec-chirho/progress-chirho.sqlite (steps_taken_chirho).
// Usage:
//   bun spec-chirho/log_step_chirho.ts start --agent <code> --action <text>   -> prints new row id
//   bun spec-chirho/log_step_chirho.ts end --id <id> --result <text> --overview <text>

import { Database } from "bun:sqlite";
import { dirname, join } from "node:path";

const db_path_chirho = join(dirname(import.meta.path), "progress-chirho.sqlite");
const db_chirho = new Database(db_path_chirho);
db_chirho.run(`
  create table if not exists steps_taken_chirho (
    id_chirho integer primary key autoincrement,
    agent_code_chirho text not null,
    timestamp_start_chirho text not null,
    timestamp_end_chirho text,
    action_taken_chirho text not null,
    result_of_action_chirho text,
    overview_of_result_chirho text
  )
`);

function arg_value_chirho(name_chirho: string): string | undefined {
  const index_chirho = Bun.argv.indexOf(name_chirho);
  return index_chirho >= 0 ? Bun.argv[index_chirho + 1] : undefined;
}

function require_arg_chirho(name_chirho: string): string {
  const value_chirho = arg_value_chirho(name_chirho);
  if (!value_chirho) {
    console.error(`missing ${name_chirho}`);
    process.exit(1);
  }
  return value_chirho;
}

const command_chirho = Bun.argv[2];
const now_chirho = new Date().toISOString();

if (command_chirho === "start") {
  const row_chirho = db_chirho
    .query(
      `insert into steps_taken_chirho (agent_code_chirho, timestamp_start_chirho, action_taken_chirho)
       values (?, ?, ?) returning id_chirho`,
    )
    .get(require_arg_chirho("--agent"), now_chirho, require_arg_chirho("--action")) as {
    id_chirho: number;
  };
  console.log(row_chirho.id_chirho);
} else if (command_chirho === "end") {
  db_chirho
    .query(
      `update steps_taken_chirho
       set timestamp_end_chirho = ?, result_of_action_chirho = ?, overview_of_result_chirho = ?
       where id_chirho = ?`,
    )
    .run(
      now_chirho,
      require_arg_chirho("--result"),
      require_arg_chirho("--overview"),
      Number(require_arg_chirho("--id")),
    );
  console.log("ok");
} else {
  console.error(
    "usage: log_step_chirho.ts start --agent A --action TEXT | end --id N --result TEXT --overview TEXT",
  );
  process.exit(1);
}
