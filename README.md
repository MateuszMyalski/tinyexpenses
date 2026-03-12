```
                                         ▗▄▄▄▖▄ ▄▄▄▄  ▄   ▄            
                                           █  ▄ █   █ █   █            
                                           █  █ █   █  ▀▀▀█            
                                           █  █       ▄   █            
                                                       ▀▀▀             


                            ▗▄▄▄▖▄   ▄ ▄▄▄▄  ▗▞▀▚▖▄▄▄▄   ▄▄▄ ▗▞▀▚▖ ▄▄▄ 
                            ▐▌    ▀▄▀  █   █ ▐▛▀▀▘█   █ ▀▄▄  ▐▛▀▀▘▀▄▄  
                            ▐▛▀▀▘▄▀ ▀▄ █▄▄▄▀ ▝▚▄▄▖█   █ ▄▄▄▀ ▝▚▄▄▖▄▄▄▀ 
                            ▐▙▄▄▖      █                               
                                       ▀                               
```

# TinyExpenses v2.0

TinyExpenses is a lightweight self-hosted home budget tracker written in Rust.
It stores data in CSV and TOML files and can be run directly or with Docker.

The goal is simple: **track income and expenses without complex accounting software.**

The budget model follows four main categories:

- Income
- Needs
- Wants
- Savings

A commonly used rule with this structure is the **50 / 30 / 20 rule**:

- 50% Needs
- 30% Wants
- 20% Savings

---

# Quick start (Docker)

```bash
git clone https://github.com/MateuszMyalski/tinyexpenses
cd tinyexpenses

mkdir logs

docker build -t tinyexpenses .

docker run \
  -v ./accounts:/app/accounts \
  -v ./logs:/app/logs \
  -e SECRET_KEY=<32-char-secret> \
  -p 8080:8080 \
  -u 1000:1000 \
  -d tinyexpenses

Then open:

http://localhost:8080
```

## Quick start (without Docker)

```bash
git clone https://github.com/MateuszMyalski/tinyexpenses
cd tinyexpenses

cargo build --release
SECRET_KEY=<32-char secret key> ./target/release/tinyexpenses --bind <IP:PORT> --db <db path>
```

---

# Features

- Yearly report with category totals and balance
- Monthly report optimized for mobile view
- Fast expense append form with search
- Budget planning (expected vs actual spending)
- Savings envelopes for tracking goals
- File-based database (CSV + TOML)
- Rolling logs for errors and suspicious activity
- API endpoint for adding expenses
- Simple account settings (currency, password, API token)

---

# Why file-based storage?

All user data is stored as plain files:

- CSV – expenses and income
- TOML – configuration

Advantages:

- easy backups (git, rsync, etc.)
- manual editing if needed
- no database server required

This also makes the system easier to audit.


# User Guide

TinyExpenses is designed to keep budgeting simple.
A typical workflow looks like this:

1. Define categories
2. Add expenses and income
3. Review reports
4. (Optional) plan your yearly budget

You should be able to maintain your budget in a few seconds per transaction.

---

# Budget structure

The system uses four main categories:

- **Income** – salary, freelance work, refunds, gifts, etc.
- **Needs** – essential expenses such as rent, food, transport, utilities
- **Wants** – non-essential spending such as restaurants, hobbies, subscriptions
- **Savings** – money set aside for long-term goals

Each entry in the system belongs to a **subcategory**, which itself belongs to one of the four main categories.

Example:

| Subcategory | Main category |
|--------------|--------------|
| Salary | Income |
| Groceries | Needs |
| Rent | Needs |
| Coffee | Wants |
| Vacation | Savings |

You define subcategories once and then reuse them when adding entries.

---

# Adding expenses or income

To add a new entry:

1. Open **Append report**
2. Select a **subcategory**
3. Enter the **date**
4. Enter the **value**
5. (Optional) add a **description**

## Rules

Each subcategory marked as an expense will decrease the account's yearly balance. Categories marked as income will increase it.

You can override this behavior by entering a negative value in the amount field (using “-”).

This is useful for cases like reimbursements. For example, if you made a purchase and later received a cashback, you can first log the expense, then log the compensation with a negative amount. The balance will reflect the net amount, while both entries remain visible in the records.

---

# Reports

TinyExpenses provides two report views.

## Yearly report

The yearly report is the main overview of your finances.

It shows:

- all transactions for the year
- totals per category
- total income
- total expenses
- current balance

This is the best place to review your full financial picture.

## Monthly report

The monthly report shows the same data but grouped by month.

It is designed for quick review and works better on mobile devices.

---

# Editing data

All expenses are stored in a CSV file.

You can modify them in two ways:

1. Using **Edit report** in the web interface
2. Editing the CSV file directly

Editing the file manually is safe because the system uses a simple file-based structure.

---

# Budget planning

The **Plans** module allows you to define expected income and expenses for the year.

The application will compare expected values vs actual values.

This makes it easy to detect:

- overspending
- unexpected expenses
- additional income

---

# Savings envelopes

Savings can be separated into logical "envelopes".

Even if all money is in one bank account, envelopes allow you to track how much is already reserved for each goal.

Savings update automatically when entries belonging to the **Savings** category are added.

---

# API usage

TinyExpenses exposes a simple API endpoint for adding expenses.

This allows integration with external tools such as:

- mobile shortcuts
- scripts
- automation tools
- personal finance bots

The endpoint requires an **API token** generated in account settings.

Example payload:

```
PUT /api/v1/<login>/report/append
```

Headers:

```
X-API-Key: <token>
Content-Type: application/json
```

Body:

```
{
    "subcategory": "Coffee",
    "date": "2026-03-12",
    "value": "-12.3",
    "description": "New beans in town!"
}
```

---

# Data storage

Each user account is stored in a directory.

Typical structure:

```
accounts/
    user1/
        year/
            categories.csv
            report.csv
            plans.csv
        savings.toml
        config.toml
```

This design allows:

- easy backups
- manual editing
- version control using git

No database server is required.