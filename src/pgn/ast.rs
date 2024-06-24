// use super::tree::Tree;
use litemap::LiteMap;
use pgn_lexer::parser::Token;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PgnToken<'a> {
    Token(Token<'a>),
    VariationPointer(u16),
    #[default]
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PgnVariation<'a>(Vec<PgnToken<'a>>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PgnGame<'a>((Vec<Token<'a>>, LiteMap<u16, PgnVariation<'a>>));

pub fn pgn_tokens_to_ast<'a>(tokens: &mut VecDeque<Token<'a>>) -> Vec<PgnGame<'a>> {
    let mut tree: Vec<PgnGame<'a>> = Vec::new();
    let mut game_number = 0;
    let mut amount_of_encountered_variations = 1;

    tree.push(PgnGame::default());
    unsafe {
        let value = &mut tree.get_unchecked_mut(0).0;

        value.0 = Vec::new();
        value.1.insert(0, PgnVariation::default());
    }

    while tokens.len() != 0 {
        next_token(
            tokens,
            &mut tree,
            &mut game_number,
            0,
            &mut amount_of_encountered_variations,
        );
    }

    tree.pop();

    tree
}

macro_rules! push_token {
    ($tree:expr, $game_number:expr, $variation_number:expr, $token:expr) => {
        $tree
            .get_mut($game_number as usize)
            .unwrap()
            .0
             .1
            .get_mut($variation_number)
            .unwrap()
            .0
            .push($token.clone())
    };
}

// TODO: AST building is currently done in plain strings only. Decide wether or not to convert it to binary format straight away, or do it with multi-threading after the fact
fn next_token<'a>(
    tokens: &mut VecDeque<Token<'a>>,
    tree: &mut Vec<PgnGame<'a>>,
    game_number: &mut u32,
    variation_number: u16,
    amount_of_encountered_variations: &mut u16,
) {
    // NOTE: I don't know if this is slow. (Like this whole approach) I'm just gonna pretend it isn't until it causes problems
    let token = unsafe { tokens.pop_front().unwrap_unchecked() };

    match token {
        Token::Move(_)
        | Token::Commentary(_)
        | Token::NAG(_)
        | Token::MoveAnnotation(_)
        | Token::MoveNumber(_, _) => {
            push_token!(
                tree,
                *game_number,
                &variation_number,
                PgnToken::Token(token)
            )
        }
        Token::TagSymbol(_) | Token::TagString(_) => tree
            .get_mut(*game_number as usize)
            .unwrap()
            .0
             .0
            .push(token),
        Token::NullMove(_) => {}
        Token::EscapeComment(_) => { /* NOTE: IDK what to do with this */ }
        Token::Result(_) => {
            tree.get_mut(*game_number as usize)
                .unwrap()
                .0
                 .0
                .push(token);

            *game_number += 1;
            *amount_of_encountered_variations = 1;

            tree.push(PgnGame::default());
            unsafe {
                let value = &mut tree.get_unchecked_mut(*game_number as usize).0;

                value.0 = Vec::new();
                value.1.insert(variation_number, PgnVariation::default());
            }
        }
        Token::StartVariation(_) => {
            let new_variation_number = *amount_of_encountered_variations * (variation_number + 1);

            *amount_of_encountered_variations += 1;

            push_token!(
                tree,
                *game_number,
                &variation_number,
                PgnToken::VariationPointer(new_variation_number)
            );

            unsafe {
                let value = &mut tree.get_unchecked_mut(*game_number as usize).0;
                value
                    .1
                    .insert(new_variation_number, PgnVariation::default());
            }

            next_token(
                tokens,
                tree,
                game_number,
                new_variation_number,
                amount_of_encountered_variations,
            );
        }
        Token::EndVariation(_) => {
            return;
        }
    }

    if tokens.len() != 0 {
        next_token(
            tokens,
            tree,
            game_number,
            variation_number,
            amount_of_encountered_variations,
        );
    }
}