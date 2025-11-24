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
