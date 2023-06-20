use serenity::model::channel::Message;
use serenity::model::prelude::ChannelId;
use serenity::client::Context;

use std::fmt::Write;
use std::error::Error;
use ndm::RollSet;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

use cpc::units::convert;
use cpc::units::Unit;

use crate::config::*;

// function for sending messages with batteries included 
async fn send(ctx: &Context, msg: &Message, cnt: &str) {
    if let Err(why) = msg.reply(ctx, cnt).await {
        eprintln!("Message Err: {:?}", why);
    }
}

pub async fn bolt_cmd(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    let conf = &mut get_config().await?.clone();
    let player_count = conf.players.len();
    
    // check if the amount of players/countries needed is satisfied
    if player_count < 2 
        || conf.countries.len() < player_count 
        || conf.countries.is_empty() 
    {
        send(&ctx, &msg, "Not Enough Players or Countries!").await;
        return Err("Not enough players or countries to start game!".into());
    }

    send(&ctx, &msg, "doin the thing!").await;

    let mut reply = "".to_string();

    // shuffle both for randomly assigning
    conf.players.shuffle(&mut thread_rng());
    conf.countries.shuffle(&mut thread_rng());

    let players_per_team = player_count / conf.teams as usize;

    for team_num in 0..conf.teams {
        writeln!(reply, "\nTeam: {}:", team_num + 1)?;

        // append players to every team
        for i in 0..players_per_team {
            writeln!(reply, "<@{}> - {}", conf.players[i], conf.countries[i])?;
            // remove those players from that team
        }
        conf.players.drain(0..players_per_team);
        conf.countries.drain(0..players_per_team);
    }

    // append the last player is there is one remaining
    if player_count % conf.teams as usize == 1{
        writeln!(reply, "<@{}> - {}",
            conf.players[0],
            conf.countries[0]
        )?;
    }

    // gen a random points number
    let points = thread_rng().gen_range(conf.min_points..=conf.max_points);

    // round to the nearest 10
    let points = (points + 5) / 10 * 10;
    writeln!(reply, "\nPoints per Team: {}", points)?;

    // send the message in the provided rolls channel
    if let Err(why) = ChannelId(conf.roll_channel).say(ctx, reply).await {
        eprintln!("Message Err: {:?}", why);
    }
    Ok(())
}

pub async fn roll_cmd(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // roll the dices, and handle error if one occurs
    match arg.parse::<RollSet>() {
       Ok(roll) => msg.reply(&ctx, format!("{}", roll)).await?,
       Err(_) => msg.reply(&ctx, "no").await?, 
    };
    Ok(())
}

pub async fn join_cmd(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    let mut conf = get_config().await?.clone();
    let user = msg.author.id.to_string();

    // if the user already joined, do nothing
    if conf.players.contains(&user) {
        send(&ctx, &msg, "Already Joined!").await;
        return Ok(());
    }

    // insert the player into config
    conf.players.insert(conf.players.len(), user); 

    // commit the changes to the global conf
    modify_config(conf).await?;

    // confirm to user
    send(&ctx, &msg, "Joined! \'!lv\' to leave").await;
    Ok(())
}

pub async fn leave_cmd(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    let mut conf = get_config().await?.clone();
    let user = msg.author.id.to_string();
    // no need to check if the player exists, cause if not it'll just do nothing anyway

    // insert the player into config
    conf.players.retain(|u| u != &user);
    
    modify_config(conf).await?;

    // confirm to user
    send(&ctx, &msg, "Left, cya!").await;
    Ok(())
}

// removes the specified player
pub async fn remove_cmd(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    let mut conf = get_config().await?.clone();
    let arg = arg.trim().replace(&['<', '>', '@'][..], "");

    // no need to check if the player exists, cause if not it'll just do nothing anyway

    // remove the player from config
    conf.players.retain(|u| u != &arg);
    
    modify_config(conf).await?;

    // confirm to user
    send(&ctx, &msg, "Left, cya!").await;
    Ok(())
}

pub async fn points_cmd(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // split and clean up the args
    let arg: Vec<&str> = arg.split_whitespace().take(2).collect();
    // check if we can even parse them
    let is_numeric: bool = arg.iter().any(|n| n.to_string().parse::<u16>().is_ok());

    // if not, let the user know they f'd up
    if arg.is_empty() || !is_numeric {
        send(&ctx, &msg, "Invalid Format!\ntry: `!points <min> <max>`").await;
        return Ok(());
    }

    // finally read the config
    let mut conf = get_config().await?.clone();

    // parse both into integers
    conf.min_points = arg[0].parse::<u16>()?;
    conf.max_points = arg[1].parse::<u16>()?;

    // modify!!!
    modify_config(conf).await?;
    
    // confirm!!
    send(&ctx, &msg, "min/max points updated!").await;
    Ok(())
}

pub async fn teams_cmd(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    // clean up
    let arg = arg.trim();

    // check if empty or if we can even parse it
    if arg.is_empty() || !arg.parse::<u8>().is_ok() {
        send(&ctx, &msg, "Invalid Format!\ntry: `!teams <int>`").await;
        return Ok(());
    }

    let mut conf = get_config().await?.clone();

    // parse parse parse! im gonna step on the parse
    conf.teams = arg.parse::<u8>()?;

    // chang
    modify_config(conf).await?;
    
    // confirm
    send(&ctx, &msg, "Teams updated!").await;
    Ok(())
}

pub async fn ls_cmd(ctx: &Context, msg: &Message) -> Result<(), Box<dyn Error>> {
    // get conf as immutable
    let conf = get_config().await?;

    // check if there are any players
    if conf.players.is_empty() {
        send(&ctx, &msg, "No players!").await;
        return Ok(());
    }

    // list them all
    let mut reply = "Players:\n".to_string();
    for (i, player) in conf.players.iter().enumerate() {
        writeln!(reply, "{}. <@{}>", i + 1, player)?;
    } 

    // do the points too
    writeln!(reply, "\nPoints:\nmin: {}\nmax: {}", conf.min_points, conf.max_points)?;
    
    // send the list of players
    send(&ctx, &msg, &reply).await;
    Ok(())
}

pub async fn convert_cmd(ctx: &Context, msg: &Message, arg: &str) -> Result<(), Box<dyn Error>> {
    let args = arg.split_once(">").unwrap_or(("", ""));

    let num = cpc::eval(args.0, true, Unit::Kelvin, false)?;

    let to_unit = match args.1.trim().to_lowercase().as_str() {
        "ns" => Unit::Nanosecond,
        //"" => Unit::Microsecond,
        "ms" => Unit::Millisecond,
        "s" => Unit::Second,
        "min" => Unit::Minute,
        "h" => Unit::Hour,
        /*"" => Unit::Day,
        "" => Unit::Week,
        "" => Unit::Month,*/
        "y" => Unit::Year,
        "mm" => Unit::Millimeter,
        "cm" => Unit::Centimeter,
        "dm" => Unit::Decimeter,
        "m" => Unit::Meter,
        "km" => Unit::Kilometer,
        "in" => Unit::Inch,
        "ft" => Unit::Foot,
        "yd" => Unit::Yard,
        "mi" => Unit::Mile,
        "ly" => Unit::LightYear,
        "ls" => Unit::LightSecond,
        /*"" => Unit::SquareMillimeter,
        "" => Unit::SquareCentimeter,
        "" => Unit::SquareDecimeter,
        "" => Unit::SquareMeter,
        "" => Unit::SquareKilometer,
        "" => Unit::SquareInch,
        "" => Unit::SquareFoot,
        "" => Unit::SquareYard,
        "" => Unit::SquareMile,
        "" => Unit::Are,
        "" => Unit::Decare,
        "" => Unit::Hectare,
        "" => Unit::Acre,
        "" => Unit::CubicMillimeter,
        "" => Unit::CubicCentimeter,
        "" => Unit::CubicDecimeter,
        "" => Unit::CubicMeter,
        "" => Unit::CubicKilometer,
        "" => Unit::CubicInch,
        "" => Unit::CubicFoot,
        "" => Unit::CubicYard,
        "" => Unit::CubicMile,*/
        "ml" => Unit::Milliliter,
        "cl" => Unit::Centiliter,
        "dl" => Unit::Deciliter,
        "l" => Unit::Liter,
        //"" => Unit::Teaspoon,
        //"" => Unit::Tablespoon,
        "fl oz" => Unit::FluidOunce,
        //"" => Unit::Cup,
        //"" => Unit::Pint,
        //"" => Unit::Quart,
        //"" => Unit::Gallon,
        "mg" => Unit::Milligram,
        "g" => Unit::Gram,
        "hg" => Unit::Hectogram,
        "kg" => Unit::Kilogram,
        "t" => Unit::MetricTon,
        "oz" => Unit::Ounce,
        "lb" => Unit::Pound,
        "st" => Unit::Stone,
        "j" => Unit::Joule,
        "nm" => Unit::NewtonMeter,
        "kj" => Unit::Kilojoule,
        "mj" => Unit::Megajoule,
        "gj" => Unit::Gigajoule,
        "tj" => Unit::Terajoule,
        "cal" => Unit::Calorie,
        "kcal" => Unit::KiloCalorie,
        "btu" => Unit::BritishThermalUnit,
        "wh" => Unit::WattHour,
        "kwh" => Unit::KilowattHour,
        "mwh" => Unit::MegawattHour,
        "gwh" => Unit::GigawattHour,
        "twh" => Unit::TerawattHour,
        "pwh" => Unit::PetawattHour,
        "w" => Unit::Watt,
        "kw" => Unit::Kilowatt,
        "mw" => Unit::Megawatt,
        "gw" => Unit::Gigawatt,
        "tw" => Unit::Terawatt,
        "pw" => Unit::Petawatt,
        "btu/min" => Unit::BritishThermalUnitsPerMinute,
        "btu/h" => Unit::BritishThermalUnitsPerHour,
        "hp" => Unit::MetricHorsepower,
        "ma" => Unit::Milliampere,
        "a" => Unit::Ampere,
        "ka" => Unit::Kiloampere,
        //"" => Unit::Abampere,
        //"" => Unit::Milliohm,
        //"" => Unit::Ohm,
        //"" => Unit::Kiloohm,
        "mV" => Unit::Millivolt,
        "v" => Unit::Volt,
        "kv" => Unit::Kilovolt,
        "pa" => Unit::Pascal,
        "kpa" => Unit::Kilopascal,
        "atm" => Unit::Atmosphere,
        "mb" => Unit::Millibar,
        "b" => Unit::Bar,
        "inhg" => Unit::InchOfMercury,
        "lbf/in2"|"psi" => Unit::PoundsPerSquareInch,
        "torr" => Unit::Torr,
        "hz" => Unit::Hertz,
        "khz" => Unit::Kilohertz,
        "mhz" => Unit::Megahertz,
        "ghz" => Unit::Gigahertz,
        "thZ" => Unit::Terahertz,
        "phz" => Unit::Petahertz,
        "rpm" => Unit::RevolutionsPerMinute,
        "km/h" => Unit::KilometersPerHour,
        "m/s" => Unit::MetersPerSecond,
        "mi/h" => Unit::MilesPerHour,
        "ft/s" => Unit::FeetPerSecond,
        "kn"|"kt" => Unit::Knot,
        "k" => Unit::Kelvin,
        "c" => Unit::Celsius,
        "f" => Unit::Fahrenheit,
        &_ => Unit::NoUnit,
    };

    let conv = match convert(num, to_unit) {
        Ok(number) => number,
        Err(why) => {
            msg.reply(&ctx, format!("{}", why)).await?;
            return Ok(());
        },
    };

    msg.reply(&ctx, format!("{:.5}{}", conv.value, args.1.trim())).await?;
    Ok(())
}

