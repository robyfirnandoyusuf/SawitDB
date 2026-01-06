# 🌴 SawitDB
Built with ❤️ greycat using Rust
<hr>
<p>SawitDB is a JSON-style document database written in Rust, built as a technical experiment and a satirical commentary on Indonesian government systems particularly the bureaucratic mindset and the brutal expansion of oil palm plantations in Aceh, Sumatra.</p>
<p>SawitDB is a lightweight, JSON-based, document-oriented database that runs as a TCP server and can be accessed via a custom URI scheme:</p>

```
sawit://host:port/database
```


It aims to feel like MongoDB, without pretending to be MongoDB.
### Philosophy
This project exists at the intersection of software engineering and political satire.

SawitDB uses the familiar shape of a document database to mirror how large-scale palm oil expansion is often managed: rapid execution, minimal structure, questionable validation, and heavy reliance on paperwork rather than long-term sustainability.

It is not an attack on individuals, but a critique of systems, incentives, and governance patterns that prioritize economic output over environmental and social impact.

### Features:
- Multi-database (each database is a folder)
- Document-oriented (JSON documents, Mongo-style)
- TCP server (async, Tokio-based)
- Custom binary framing (length-prefixed JSON)
- Mongo-like experience
  - databases
  - collections
  - documents
  - insert / find / update / delete
- CLI included
- 🦀 Written in Rust

## 🚀 Getting Started
### 1. Start the Server
<img width="622" height="387" alt="image" src="https://github.com/user-attachments/assets/8b74c4fc-77d0-4442-9b28-cde40d0ff00b" />
<br>

```bash
cargo run -- serve --addr 127.0.0.1:27017 --data-dir data
```
This will start the SawitDB daemon.

### 2. Use the CLI
#### Ping server
<img width="597" height="225" alt="image" src="https://github.com/user-attachments/assets/daa4286c-015d-492a-8538-8e0c0ea588bf" />
<br>

```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 ping
```
#### Create database
<img width="632" height="342" alt="image" src="https://github.com/user-attachments/assets/eca08e8f-cf9a-4d7b-b055-88a2caafa62f" />
<br>

```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 db-create toko
```
#### Insert document
```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 insert toko users '{"name":"Budi","age":20}'
```
#### Find documents
```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 find toko users "name=Budi"
```
#### Update document (by id)
```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 update toko users <ID> '{"age":21}'
```

#### Delete document
```bash
cargo run -- cli --uri sawit://127.0.0.1:27017 delete toko users <ID>
```
### Database Operations
<table>
  <tr>
    <th>
      Operation
    </th>
    <th>
      Command
    </th>
  </tr>
<tr>
  <td>List DBs</td>
  <td>db-list</td>
</tr>	
<tr>
  <td>Create DB</td>
  <td>db-create <name></td>
</tr>
<tr>
  <td>Rename DB</td>
  <td>db-rename <old> <new></td>
</tr>
<tr>
  <td>Drop DB	</td>
  <td>db-drop <name></td>
</tr>
</table>

### Connection URI
SawitDB uses a Mongo-like URI syntax:

```bash
sawit://username:password@host:port/database
```

Example:
```arduino
sawit://127.0.0.1:27017/toko
```
⚠️ This is NOT MongoDB wire protocol.
It only looks familiar so developers feel at home.
