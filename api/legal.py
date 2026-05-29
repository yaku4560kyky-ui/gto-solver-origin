from pydantic import BaseModel
from fastapi import APIRouter


router = APIRouter(prefix="/api/v1/legal", tags=["legal"])


class LegalSection(BaseModel):
    title: str
    body: str


class LegalDocument(BaseModel):
    title: str
    jurisdiction: str
    language: str
    effective_date: str
    sections: list[LegalSection]


@router.get("/terms", response_model=LegalDocument)
def terms() -> LegalDocument:
    return LegalDocument(
        title="利用規約",
        jurisdiction="Japan",
        language="ja",
        effective_date="2026-05-30",
        sections=[
            LegalSection(
                title="サービス内容",
                body="本サービスはポーカー学習および戦略分析を目的としたソフトウェアです。",
            ),
            LegalSection(
                title="禁止事項",
                body="違法賭博、第三者の権利侵害、不正アクセス、過度な自動化利用を禁止します。",
            ),
            LegalSection(
                title="免責",
                body="解析結果の完全性、収益性、特定目的への適合性を保証しません。",
            ),
        ],
    )


@router.get("/tokushouhou", response_model=LegalDocument)
def tokushouhou() -> LegalDocument:
    return LegalDocument(
        title="特定商取引法に基づく表記",
        jurisdiction="Japan",
        language="ja",
        effective_date="2026-05-30",
        sections=[
            LegalSection(title="販売事業者", body="GTO Solver Demo"),
            LegalSection(title="運営責任者", body="Demo Operator"),
            LegalSection(title="所在地", body="請求があった場合、遅滞なく開示します。"),
            LegalSection(title="連絡先", body="support@example.com"),
            LegalSection(title="販売価格", body="各購入画面に税込価格を表示します。"),
            LegalSection(
                title="返品・キャンセル",
                body="デジタル商品の性質上、提供開始後の返品は原則として受け付けません。",
            ),
        ],
    )


@router.get("/privacy", response_model=LegalDocument)
def privacy() -> LegalDocument:
    return LegalDocument(
        title="プライバシーポリシー",
        jurisdiction="Japan",
        language="ja",
        effective_date="2026-05-30",
        sections=[
            LegalSection(
                title="取得する情報",
                body="アカウント情報、利用ログ、問い合わせ内容、決済に必要な情報を取得する場合があります。",
            ),
            LegalSection(
                title="利用目的",
                body="サービス提供、本人確認、不正利用防止、品質改善、法令対応に利用します。",
            ),
            LegalSection(
                title="第三者提供",
                body="法令に基づく場合を除き、本人の同意なく個人情報を第三者へ提供しません。",
            ),
        ],
    )
