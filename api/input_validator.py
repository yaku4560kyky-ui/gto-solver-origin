from fastapi import HTTPException, status


def validate_card(card: int) -> int:
    if card < 0 or card > 51:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Card must be an integer from 0 to 51",
        )
    return card


def validate_iterations(iterations: int) -> int:
    if iterations < 1 or iterations > 10_000_000:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Iterations must be between 1 and 10000000",
        )
    return iterations


def validate_seven_cards(cards: list[int]) -> list[int]:
    if len(cards) != 7:
        raise HTTPException(status_code=400, detail="Need 7 cards")

    for card in cards:
        validate_card(card)

    if len(set(cards)) != 7:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Cards must be unique",
        )

    return cards
