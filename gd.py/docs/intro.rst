.. currentmodule:: gd

Introduction
============

gd.py is a Geometry Dash API, created to simplify interaction with GD servers.

The aim is to create a library following the best aspects of Object-Oriented Programming.

gd.py is async-ready, which means it can be easily run in parallel with other async programs.

For example, you can use it in a Discord bot:

.. code-block:: python3

    from discord.ext import commands  # import commands extension
    import discord

    import gd

    bot = commands.Bot(command_prefix="> ")
    client = gd.Client()

    @bot.event
    async def on_ready() -> None:
        bot.client = client  # attach gd.Client to commands.Bot

        activity = discord.Activity(type=discord.ActivityType.playing, name="Geometry Dash")

        await bot.change_presence(activity=activity, status=discord.Status.online)

    @bot.command(name="daily")
    async def get_daily(ctx: commands.Context) -> None:
        try:
            daily = await bot.client.get_daily()

        except gd.MissingAccess:
            # couldn"t fetch a daily level
            return await ctx.send(
                embed=discord.Embed(
                    title="Error Occured",
                    description="Failed to get a daily level.",
                    color=0xFF5555,
                )
            )

        embed = (
            discord.Embed(color=0x7289da, title="Daily", timestamp=ctx.message.created_at)
            .add_field(name="Name", value=daily.name)
            .add_field(name="Difficulty", value=f"{daily.stars} ({daily.difficulty.title})")
            .add_field(name="ID", value=f"{daily.id}")
            .set_footer(text=f"Creator: {daily.creator.name}")
        )

        await ctx.send(embed=embed)

    bot.run("BOT_TOKEN")

(You can find documentation for ``discord.py`` library `here <https://discordpy.readthedocs.io/>`_)
