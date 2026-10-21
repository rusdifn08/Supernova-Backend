# Supernova Labs Studio — Multiplatform Backend Engine
**High-Performance Enterprise Backend built with Rust & Axum**

A robust, secure, and asynchronous backend engine designed to power Supernova Labs' three flagship enterprise digital solutions:
1. **Universal Ordering & Booking System** (Sports Facilities, Photo Studios, Barbershops, PS Rentals, Gaming Cafes, Professional Services)
2. **Unified POS, ERP, CRM & Finance Suite** (Cashier POS, Inventory Control, Staff Monitoring, Customer Loyalty CRM, Profit & Loss Journal)
3. **Ordering In Table with QR Code** (Dine-in Restaurant QR Menu, Digital Variants, Realtime Kitchen Display System)
4. **Interactive Sandbox & Trial Inquiries**

---

## 🛠️ Technology Stack & Architecture

- **Language:** Rust 1.80+ (2021 Edition)
- **HTTP Web Framework:** [Axum 0.8](https://github.com/tokio-rs/axum) (Modern `{capture}` URI routing syntax)
- **Async Runtime:** [Tokio 1.54](https://tokio.rs/) (Multi-threaded Work-Stealing)
- **Database Engine:** [PostgreSQL (Neon Serverless Cloud)](https://neon.tech/)
- **Database Driver:** [SQLx 0.9](https://github.com/launchbadge/sqlx) (Pure async, TLS Rustls Ring, Parameterized Query Engine with `SqlSafeStr` static injection prevention)
- **Serialization:** Serde & Serde JSON
- **Middleware:** Tower 0.5 & Tower-HTTP 0.7 (Trace, CORS, Timeout)
- **Error Handling:** ThisError 2.0
- **Logging & Telemetry:** Tracing & Tracing-Subscriber

---

## 🔒 Security & Robustness Highlights

1. **SQL Injection Immune:** 100% of database interactions are built using SQLx parameterized statements (`$1`, `$2`, ...) and strictly conform to compile-time `SqlSafeStr` safety bounds. No dynamic raw string interpolation into SQL queries.
2. **Atomic ACID Transactions:** POS order checkouts, inventory stock deduction, and dine-in kitchen tickets execute inside dedicated database transactions (`pool.begin() ... tx.commit()`) with row-level locking (`FOR UPDATE`) to prevent race conditions.
3. **Conflict & Overlap Prevention:** The Booking Engine performs real-time timestamp collision detection (`start_time < existing.end_time AND end_time > existing.start_time`) to prevent double-booking.
4. **Safe Error Masking:** Database connection strings, credentials, and internal stack traces are caught by `AppError` and logged via `tracing::error!` on the server side, returning clean, non-leaking JSON error envelopes to clients with proper HTTP status codes.
5. **Connection Pooling:** Pool configuration handles idle connection recycling, max lifetimes, and SSL handshake requirements for serverless cloud databases.

---

## 🚀 Getting Started

### 1. Environment Configuration
Create a `.env` file in the root directory (refer to `.env.example`):
```env
DATABASE_URL=postgresql://neondb_owner:YOUR_PASSWORD@ep-rough-credit-a1xtzvbz-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require
SERVER_HOST=0.0.0.0
SERVER_PORT=8088
RUST_LOG=info,supernova_backend=debug,tower_http=info
CORS_ALLOWED_ORIGINS=http://localhost:4321,http://localhost:3000,https://supernovalabs.studio
```

### 2. Run Database Schema & Backend Server
```bash
# Development Mode (auto migrates schema & seeds on start)
cargo run

# Release Build (Optimized Binary)
cargo build --release
./target/release/supernova-backend
```

---

## 📡 API Reference & Endpoints

Base URL: `http://localhost:8088/api/v1`

### 1. Health & Status
- `GET /health` / `GET /api/v1/health`
  - Verifies server status and performs a live `SELECT 1` ping to the Neon PostgreSQL cluster.

### 2. Universal Booking Engine
- `GET /api/v1/booking/types` — List all business categories (Sports, Studio, Barber, PS, Warnet, etc.).
- `GET /api/v1/booking/resources` — List bookable venues, courts, rooms, stations, and hourly rates.
- `POST /api/v1/booking/reserve` — Create a reservation with collision detection.
  ```json
  {
    "resource_id": "11111111-1111-1111-1111-111111111111",
    "customer_name": "Fajar Pratama",
    "customer_phone": "081234567890",
    "customer_email": "fajar@example.com",
    "start_time": "2026-10-22T19:00:00Z",
    "end_time": "2026-10-22T21:00:00Z",
    "payment_method": "qris",
    "notes": "Siapkan rompi dan bola FIFA"
  }
  ```
- `GET /api/v1/booking/reservations` — List confirmed bookings.
- `PATCH /api/v1/booking/reservations/:id/status` — Update booking/payment status.

### 3. POS, ERP, CRM & Finance
- `GET /api/v1/pos/products` — Get product catalog with live stock & alerts.
- `POST /api/v1/pos/orders` — Atomic Cashier Checkout:
  - Deducts inventory stock
  - Logs audit history in ERP
  - Records revenue in Finance Journal
  - Credits loyalty points to CRM customer
  ```json
  {
    "customer_id": "bbbb1111-1111-1111-1111-111111111111",
    "payment_method": "qris",
    "items": [
      { "product_id": "aaaa1111-1111-1111-1111-111111111111", "quantity": 2 }
    ]
  }
  ```
- `GET /api/v1/crm/customers` — Get customer loyalty tiers & total spend.
- `GET /api/v1/erp/employees` — Monitor employee attendance, shift, and role.
- `GET /api/v1/finance/summary` — Real-time P&L (Total Revenue, COGS, Gross Profit, Margin %).

### 4. Ordering In Table with QR Code
- `GET /api/v1/qr-order/table/:qr_token` — Scanned by customer at the table; returns table details and digital menu with custom variants.
- `POST /api/v1/qr-order/order` — Place customer order directly from table:
  ```json
  {
    "table_qr_token": "tbl_tok_01_supernova_demo",
    "customer_name": "Reza",
    "payment_method": "qris",
    "items": [
      {
        "menu_item_id": "eeee1111-1111-1111-1111-111111111111",
        "quantity": 1,
        "selected_variant": "Medium Well, Pedas Sedang",
        "special_notes": "Tanpa bawang goreng"
      }
    ]
  }
  ```
- `GET /api/v1/qr-order/kitchen-orders` — Kitchen Display System (KDS) live ticket feed.
- `PATCH /api/v1/qr-order/orders/:id/status` — Kitchen staff updates status (`cooking` &rarr; `served` &rarr; `completed`).
- `GET /api/v1/qr-order/tables` — List restaurant tables and QR tokens.

### 5. Trial / Demo Inquiries
- `POST /api/v1/trial/inquire` — Submit live demo / trial inquiry from prospective business clients.
- `GET /api/v1/trial/inquire` — List trial submissions for sales follow-up.
