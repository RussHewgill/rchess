
use rchess_engine_lib::types::Color;

use crate::tuner_types::{MatchResult, DrawType, WinLossType};

use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{one_of,alphanumeric1},
};

impl MatchResult {
    pub fn parse(input: &str) -> Option<Self> {
        if let Ok((_,draw)) = parse_draw(&input) {
            Some(MatchResult::Draw(draw))
        } else if let Ok((_,(side, winloss))) = parse_winloss(&input) {
            Some(MatchResult::WinLoss(side, winloss))
        } else {
            None
        }
    }
}

fn parse_winloss(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,(side,winloss)) = nom::branch::alt((
        parse_adjudication_win,
        parse_mate,
        parse_loss_time,
        parse_loss_illegal_move,
        parse_loss_stall,
        parse_loss_disconnect,
    ))(s)?;
    Ok((s, (side, winloss)))
}

fn parse_draw(s: &str) -> IResult<&str, DrawType> {
    let (s,draw) = nom::branch::alt((
        parse_adjudication_draw,
        parse_repetition,
        parse_stalemate,
        parse_50move,
    ))(s)?;
    Ok((s,draw))
}



fn parse_mate(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag(" mates")(s)?;
    Ok((s, (side, WinLossType::Checkmate)))
}

fn parse_loss_time(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag(" loses on time")(s)?;
    Ok((s, (!side, WinLossType::Time)))
}

fn parse_loss_illegal_move(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag(" makes an illegal move")(s)?;
    Ok((s, (!side, WinLossType::IllegalMove)))
}

fn parse_loss_stall(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag("'s connection stalls")(s)?;
    Ok((s, (!side, WinLossType::Stalled)))
}

fn parse_loss_disconnect(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag(" disconnects")(s)?;
    Ok((s, (!side, WinLossType::Disconnect)))
}

fn parse_adjudication_win(s: &str) -> IResult<&str, (Color, WinLossType)> {
    let (s,side) = parse_color(s)?;
    let (s,_) = tag(" wins by adjudication")(s)?;
    Ok((s, (side, WinLossType::AdjudicationSyzygy)))
}

fn parse_repetition(s: &str) -> IResult<&str, DrawType> {
    let (s,_) = tag("Draw by 3-fold repetition")(s)?;
    Ok((s, DrawType::Repetition))
}

fn parse_stalemate(s: &str) -> IResult<&str, DrawType> {
    let (s,_) = tag("Draw by stalemate")(s)?;
    Ok((s, DrawType::Stalemate))
}

fn parse_50move(s: &str) -> IResult<&str, DrawType> {
    let (s,_) = tag("Draw by fifty moves rule")(s)?;
    Ok((s, DrawType::FiftyMoveRule))
}

fn parse_adjudication_draw(s: &str) -> IResult<&str, DrawType> {
    let (s,_) = tag("Draw by adjudication: SyzygyTB")(s)?;
    Ok((s, DrawType::Adjudication))
}

fn parse_color(s: &str) -> IResult<&str, Color> {
    let (s,side) = nom::branch::alt((
        tag("White"),
        tag("Black"),
    ))(s)?;
    let side = match side {
        "White" => Color::White,
        "Black" => Color::Black,
        _       => panic!(),
    };
    Ok((s,side))
}

