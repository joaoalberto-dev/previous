#!/usr/bin/env node

// The compliler will handle messages.
// A Message is a struct made of fields with specific types
//
// message User {
//   string name
//   string email
//   int    age
// }
//
// message Users {
//    [User] users
// }
//
// The compiler will then generate the code for both the client and the server
//
// const user = new User()
// user.setField('name', "John")
// user.setField('email', "john@example.com")
// user.setField('age', 30)
//
// const users = new Users()
// users.setField('users', [user])
//
// The compiler can then generate a Response
// A Response is a Either type and can be a Success or Error
//
// const response = new Response(users)
//
// This generates the following response:
// data:
//   users: [["John", "john@example.com", 30]]
// error:
//   NULL

import * as fs from "node:fs";
import * as path from "node:path";
import * as process from "node:process";
import * as util from "node:util";

const ERROR = util.styleText(["bgRed", "bold"], "Error:");
const INFO = util.styleText(["bgBlueBright", "bold"], "Info:");

function main(file_path?: string) {
  if (!file_path) {
    process.stdout.write(`${ERROR} Argument 'path' is required.\n`);
    return;
  }

  const current_dir = process.cwd();
  const final_path = path.join(current_dir, file_path);

  if (!fs.existsSync(final_path)) {
    process.stdout.write(
      `${ERROR} Directory of file ${final_path} not found.\n`
    );
    return;
  }

  const dir = fs.readdirSync(path.join(current_dir, file_path));

  for (let entry of dir) {
    const file_path = path.join(final_path, entry);
    const is_dir = fs.lstatSync(file_path).isDirectory();
    const type = is_dir ? "directory" : "file";

    process.stdout.write(`${INFO} ${type} ${entry} found.\n`);
  }
}

export { main };
