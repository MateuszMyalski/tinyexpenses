import os
import click

from tinyexpenses.models.file import DbFile
from tinyexpenses.models.accounts import ConfigCreator


@click.group()
def main():
    """TinyExpenses CLI tool."""
    pass


@main.command("init-user")
@click.argument("users_root", type=click.Path(exists=True, file_okay=False))
@click.option(
    "--username",
    prompt=True,
)
@click.option("--full-name", prompt="Full name")
@click.option("--currency", prompt=True, default="$")
@click.option("--password", prompt=True, hide_input=True, confirmation_prompt=True)
def init_user(users_root, username, full_name, currency, password):
    """Create a new user folder inside USERS_ROOT."""
    user_dir = os.path.join(users_root, username)
    config_path = os.path.join(user_dir, ConfigCreator.CONFIG_FILE_NAME)

    if os.path.exists(config_path):
        click.echo(f"❌ User already exists at {config_path}")
        return

    os.makedirs(user_dir, exist_ok=True)

    db_file = DbFile(config_path)
    config = ConfigCreator(db_file)

    config.set_username(username)
    config.set_full_name(full_name)
    config.set_currency(currency)
    config.set_password(password)
    token = config.generate_api_token()

    click.echo(f"✅ User '{username}' initialized at {config_path}")
    click.echo(f"🔑 API token: {token}")


@main.command("reset-password")
@click.argument("users_root", type=click.Path(exists=True, file_okay=False))
@click.argument("username", type=str)
@click.option("--new-password", prompt=True, hide_input=True, confirmation_prompt=True)
def reset_password(users_root, username, new_password):
    """Reset a user's password inside USERS_ROOT."""
    user_dir = os.path.join(users_root, username)
    config_path = os.path.join(user_dir, ConfigCreator.CONFIG_FILE_NAME)

    if not os.path.exists(config_path):
        click.echo(f"❌ Error loading users - does not exists at {config_path}.")
        return

    db_file = DbFile(config_path)
    config = ConfigCreator(db_file)

    config.set_password(new_password)

    click.echo(f"✅ Password reset for user '{username}'")


if __name__ == "__main__":
    main()
