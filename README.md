# Bus 🚌

Bus is a powerful and efficient server+CLI tool that creates a message queue to centralize all microservices accesses in a single place. Its primary goal is to secure, optimize, and easily manage the access control of your microservices, both internal and external.

Designed with simplicity and performance in mind, Bus uses a straightforward declarative language for declaring access control, making it an ideal choice for infrastructure and security engineers, CTOs, and tech leaders who value easily auditable, secure, and scalable solutions.

🚧 This is a work-in-progress project. Please, reach out at help@abstra.app before using it in production.

## Features

- **Easily Auditable**: All access logs are concentrated in a single source, making auditing a breeze.
- **Super Secure**: Bus restricts API access in a whitelist manner, ensuring maximum security.
- **Fast Configuration**: Its simple syntax and centralized architecture allow for quick and easy configurations.
- **Super Scalable**: Built with Rust and incorporating numerous performance optimizations, Bus is designed to scale with your needs.

## Example

Here's a simple example of Bus's declarative language:

```bus
role frontend
role backend

request new-user {
    name: string
    email: string
}

response new-user {
    uuid: string
}

allow frontend request new-user
allow backend response new-user

allow frontend listen user-created
allow backend broadcast user-created {
    id: uuid
    created: datetime
}
```

## CLI Usage

Bus CLI allows you to manage your Bus server and perform various tasks:

### Start a server

```bash
> bus serve --policy ./my-policy.bus
listening http://localhost:3030
```

This command starts a server with the policies specified in the provided `.bus` file.

### Generate a token

```bash
> bus generate-token --role service-1
e1375bc9-0708-4eb9-b3d6-2c46398d2da9
```

This command generates a token to be used by the APIs.

### Generate types

```ts
> bus generate-types --language typescript
type UserCreated = {
    id: string;
    created: Date;
}
```

## SDK

Bus provides SDKs for popular programming languages, including JavaScript, Python, Go, Java, C#, and more. Here's an example of how to use the Python SDK:

```python
from abstra.bus import listen

@listen("user-created")
def handler(ctx, evt):
    print(evt.id)
    print(evt.created)
```

## Installation

_TODO: Add instructions for installing the CLI and SDKs._

## Contributing

We welcome contributions to Bus! Please see our [contributing guidelines](CONTRIBUTING.md) for more information.

## License

Bus is licensed under the [MIT License](LICENSE).