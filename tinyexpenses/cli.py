import os
import click

from tinyexpenses.models.accounts import Config


@click.group()
def main():
    """TinyExpenses CLI tool."""
    pass


@main.command("init-user")
@click.argument("users_root", type=click.Path(exists=True, file_okay=False))
@click.option("--username", prompt=True)
@click.option("--full-name", prompt="Full name")
@click.option("--password", prompt=True, hide_input=True, confirmation_prompt=True)
def init_user(users_root, username, full_name, password):
    """Create a new user folder inside USERS_ROOT."""
    user_dir = os.path.join(users_root, username)

    if Config.config_file_exists(user_dir):
        click.echo(f"❌ User already exists at {user_dir}")
        return

    os.makedirs(user_dir, exist_ok=True)

    Config.config_file_create_empty(user_dir)

    config = Config(user_dir)

    config.set_username(username)
    config.set_full_name(full_name)
    config.set_password(password)

    click.echo(f"✅ User '{username}' initialized at {user_dir}")


@main.command("reset-password")
@click.argument("users_root", type=click.Path(exists=True, file_okay=False))
@click.argument("username", type=str)
@click.option("--new-password", prompt=True, hide_input=True, confirmation_prompt=True)
def reset_password(users_root, username, new_password):
    """Reset a user's password inside USERS_ROOT."""
    user_dir = os.path.join(users_root, username)

    if not Config.config_file_exists(user_dir):
        click.echo(f"❌ Error: user config does not exist at {user_dir}.")
        return

    config = Config(user_dir)
    config.set_password(new_password)

    click.echo(f"✅ Password reset for user '{username}'")


if __name__ == "__main__":
    main()
