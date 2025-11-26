# Previous

Is a binary protocol and BFF framework.

Previous is made of a Schema definition, a compiler, client and server utils to help you create complete solutions with one tool.

## Design

Previous is built around `Resource`'s.

Resources are a way to describe the data your server and client can interchange.

### Anatomy of a Resource schema

You can create your Resources by defining its shape with field types and names.

```
resource User {
    string   name
    string   email
    optional number age
    bool     active
}

resource Names {
    list string name
}

resource Users {
    list User
}

resource Settings {
    nullable bool notifications
}

resource Notification {
    number default(10) interval
}
```

This will then be translated to both server and client code after the schema compilation

Server:
```js
// You can create the Resource with the expected fields
const user = new User({
    name:   'Jhon',
    email:  'jhon@email.com',
    age:    30,
    active: false
})

// Or build then in steps
const user = new User()
user.setField('name', 'Jhon')
user.setField('email', 'jhon@email.com')
user.setField('age', 30)
user.setField('active', false)

// Then send it to the client
app.get(`/user/:id`, async () => {
    // ... user creation
    return user.build()
})
```

Client:
```
import { user } from 'previous/generated/client'
const response = await fetch(`/user/1`).then(user.handle)

console.log(response.error)                   // null
console.log(response.data)                    // binary user data not serialized to json
console.log(response.data.getField('name'))   // Lazy parsed user name
console.log(response.data.getField('email'))  // Lazy parsed user email
console.log(response.data.getField('age'))    // Lazy parsed user age
console.log(response.data.getField('active')) // Lazy parsed user active
console.log(response.data.toJson())           // Lazy json parsing
```