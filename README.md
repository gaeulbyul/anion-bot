# Ani-ON 봇 [@ani_on_bot](https://telegram.me/ani_on_bot)

**Ani-ON 봇**은 애니편성표 봇입니다. 데이터는 [애니시아](http://anissia.net/)에서 가져옵니다.

모바일 웹용 애니편성표: [Ani-ON](https://anion.herokuapp.com/)

## 사용법

* `/list`: 오늘의 목록
* `/list {요일}`: 요일별 목록 (요일 = 일월화수목금토)
* `/cap {ID}`: 자막 목록 (ID = 애니별 고유번호 (#xxxx))
* `/start` OR `/help`: 도움말 메시지 출력

## 사용기술

* [Rust](https://www.rust-lang.org)
* [Iron Framework](http://ironframework.io)
* [telegram-bot](https://github.com/LukasKalbertodt/telegram-bot)

## 라이센스

MIT
