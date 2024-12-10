## Magic Eden Claimer

## English

### Socials

Telegram channel: [https://t.me/fragment_software](https://t.me/fragment_software)

Telegram chat: [https://t.me/fragment_software_chat](https://t.me/fragment_software_chat)

### Installation

#### Prerequisites

- **Rust** : Ensure you have Rust installed. You can download and install Rust from _[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)_.

### Build

Clone the repository and build the project:

```
git clone https://github.com/Fragment-Software/me-claimer.git
cd me-claimer
cargo build --release
```

### Running

1. In the browser, open this link using a proxy (required!!!) which you will use in the claimer:

`https://mefoundation.com/api/trpc/ixs.newClaimBatch?batch=1&input={"0":{"json":{"claimWallet":"veTbq5fF2HWYpgmkwjGKTYLVpY6miWYYmakML7R7LRf","allocationEvent":"tge-airdrop-final","ns":"acAvyneD7adS3yrXUp41c1AuoYoYRhnjeAWH9stbdTf","enableStake":false,"priorityFeeMicroLamports":80000}}}`

2. Navigate to the URL again (do not refresh the page; simply paste the URL into the address bar and open it again).

3. Open the browser developer tools:
   - macOS: `cmd + options (alt) + I`;
   - Linux: `ctrl + alt + I`;
   - Windows: `F12`

4. Go to the "Application" tab in the developer tools:
  1. On the left-hand side, open Cookies.
  2. Select https://mefoundation.com.
  3. Locate the cookie named cf_clearance and disable the HttpOnly flag:

5. Go to the "Console" tab in the developer tools and run the following script:
```javascript
await navigator.userAgentData.getHighEntropyValues([
    'architecture',
    'bitness',
    'model',
    'platform',
    'platformVersion',
    'uaFullVersion',
    'fullVersionList'
]).then((hints) => {
    console.log({
        'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
        'accept-language': 'ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7',
        'cache-control': 'no-cache',
        'cookie': document.cookie,
        'pragma': 'no-cache',
        'sec-ch-ua': navigator.userAgentData.brands
            .map(brand => `"${brand.brand}";v="${brand.version}"`)
            .join(', '),
        'sec-ch-ua-arch': `"${hints.architecture}"`,
        'sec-ch-ua-bitness': `"${hints.bitness}"`,
        'sec-ch-ua-full-version': `"${hints.uaFullVersion}"`,
        'sec-ch-ua-full-version-list': hints.fullVersionList
            .map(brand => `"${brand.brand}";v="${brand.version}"`)
            .join(', '),
        'sec-ch-ua-mobile': `?${navigator.userAgentData.mobile ? 1 : 0}`,
        'sec-ch-ua-model': `"${hints.model}"`,
        'sec-ch-ua-platform': `"${hints.platform}"`,
        'sec-ch-ua-platform-version': `"${hints.platformVersion}"`,
        'sec-fetch-dest': 'document',
        'sec-fetch-mode': 'navigate',
        'sec-fetch-site': 'none',
        'sec-fetch-user': '?1',
        'upgrade-insecure-requests': '1',
        'user-agent': navigator.userAgent
    });
}).catch(console.error);
```

6. Copy the resulting object:
   1. Right-click on the logged object in the console.
   2. Select Copy object.

7. Paste the copied object into data/headers.json.

Fill files in data/ directory

Execute the built binary:

`cargo run --release`

## Русский

### Наши ресурсы

Telegram channel: [https://t.me/fragment_software](https://t.me/fragment_software)

Telegram chat: [https://t.me/fragment_software_chat](https://t.me/fragment_software_chat)

### Установка

#### Предварительные требования

- **Rust** : Убедитесь, что Rust установлен. Вы можете скачать и установить Rust с [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

### Сборка

Клонируйте репозиторий и соберите проект:

```
git clone https://github.com/Fragment-Software/me-claimer.git
cd me-claimer
cargo build --release
```

### Запуск

1. В браузере эту ссылку открыть используя прокси (обязательно!!!) которую будете использовать в клеймере:

`https://mefoundation.com/api/trpc/ixs.newClaimBatch?batch=1&input={"0":{"json":{"claimWallet":"veTbq5fF2HWYpgmkwjGKTYLVpY6miWYYmakML7R7LRf","allocationEvent":"tge-airdrop-final","ns":"acAvyneD7adS3yrXUp41c1AuoYoYRhnjeAWH9stbdTf","enableStake":false,"priorityFeeMicroLamports":80000}}}`

2. После снова переходим по этой ссылке (НЕ ОБНОВЛЯЕМ СТРАНИЦУ, А СНОВА ВСТАВЛЯЕМ ЭТУ ССЫЛКУ)

3. Открываем девтулз:
   - macOS: `cmd + options (alt) + I`;
   - Linux: `ctrl + alt + I`;
   - Windows: `F12`

4. Переходим на вкладку Application -> слева на вкладке открываем Cookies -> https://mefoundation.com -> на куки cf_clearance снимаем галочку под HttpOnly (кликнуть 2 раза, чтобы пропала галочка)

5. Открываем вкладку Console -> вставляем скрипт:
```javascript
await navigator.userAgentData.getHighEntropyValues([
    'architecture',
    'bitness',
    'model',
    'platform',
    'platformVersion',
    'uaFullVersion',
    'fullVersionList'
]).then((hints) => {
    console.log({
        'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
        'accept-language': 'ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7',
        'cache-control': 'no-cache',
        'cookie': document.cookie,
        'pragma': 'no-cache',
        'sec-ch-ua': navigator.userAgentData.brands
            .map(brand => `"${brand.brand}";v="${brand.version}"`)
            .join(', '),
        'sec-ch-ua-arch': `"${hints.architecture}"`,
        'sec-ch-ua-bitness': `"${hints.bitness}"`,
        'sec-ch-ua-full-version': `"${hints.uaFullVersion}"`,
        'sec-ch-ua-full-version-list': hints.fullVersionList
            .map(brand => `"${brand.brand}";v="${brand.version}"`)
            .join(', '),
        'sec-ch-ua-mobile': `?${navigator.userAgentData.mobile ? 1 : 0}`,
        'sec-ch-ua-model': `"${hints.model}"`,
        'sec-ch-ua-platform': `"${hints.platform}"`,
        'sec-ch-ua-platform-version': `"${hints.platformVersion}"`,
        'sec-fetch-dest': 'document',
        'sec-fetch-mode': 'navigate',
        'sec-fetch-site': 'none',
        'sec-fetch-user': '?1',
        'upgrade-insecure-requests': '1',
        'user-agent': navigator.userAgent
    });
}).catch(console.error);
```

6. Копируем объект (пкм -> Copy object)

7. Вставляем в data/headers.json

Заполните файлы в data/

Запустите собранный бинарный файл:

`cargo run --release `
