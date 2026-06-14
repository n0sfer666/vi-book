# Разработка плагинов для vi-book

vi-book настраивается и расширяется через **Lua**. Это полный справочник для
авторов плагинов: модель рантайма, весь API `vibook`, события и опции, палитра
подсветки синтаксиса и построчный разбор встроенного плагина `links`.

> English version: [`plugin-development.md`](plugin-development.md).

---

## 1. Как загружаются плагины

При старте vi-book создаёт один песочничный Lua-рантайм и выполняет по порядку:

1. **Прелюдию** — определяет таблицу `vibook` и хелперы.
2. **Дефолтные плагины**, вкомпилированные в бинарь (сейчас `links`; см. §8).
3. Ваш **пользовательский конфиг**: `~/.config/vi-book/init.lua`
   (путь переопределяется переменной окружения `VIBOOK_CONFIG`).

Поскольку ваш конфиг выполняется *после* дефолтных плагинов, вы можете менять
их поведение через опции (например, отключить дефолтный плагин) и добавлять
свои команды, кеймапы, цвета палитры и обработчики событий поверх.

Конфиг — это просто Lua-скрипт. Отдельного манифеста нет: вызывайте функции
`vibook.*`, описанные ниже, и они применяются сразу.

```lua
-- ~/.config/vi-book/init.lua
vibook.opt.leader = " "
vibook.command("hello", function() vibook.buf.set_text("hello!\n") end)
```

---

## 2. Песочница

Рантайм намеренно маленький и безопасный:

- **Доступны** стандартные библиотеки: `string`, `table`, `math`, `utf8`.
- **Удалены** по умолчанию: `io`, `os`, `debug`, `package`, а также загрузчики
  `dofile`, `loadfile`, `load`/`loadstring`. Доступа к сети нет.
- `print` и `warn` равны `nil` — они испортили бы TUI. Выводите информацию
  через буфер редактора или команды.
- Ошибка плагина показывается в строке статуса; она **никогда** не роняет
  vi-book.
- Долгие скрипты прерываются по бюджету инструкций, поэтому случайный
  бесконечный цикл не подвесит event loop.

Если будущая возможность требует файловой системы или процесса, vi-book
предоставляет под неё *узкую специализированную* функцию (например,
`vibook.open_external`, §6), а не выдаёт плагину сырой `os.execute`.

---

## 3. Опции — `vibook.opt`

| Опция | Тип | По умолчанию | Назначение |
|-------|-----|--------------|------------|
| `leader` | string | `" "` | Клавиша, открывающая which-key меню. |
| `insert_escape` | string | `"jj"` | Набор для выхода из edit в normal; `""` отключает. |
| `menu_position` | string | `"bottom"` | Нижняя панель `"bottom"` или модалка `"center"`. |
| `code_editor` | string | `""` | Внешний редактор для код-ссылок; fallback `$EDITOR`. |
| `plugins` | table | `{}` | Тогглы плагинов, напр. `{ links = false }`. |

```lua
vibook.opt.leader = ","
vibook.opt.insert_escape = "jk"
vibook.opt.menu_position = "center"
vibook.opt.code_editor = "nvim"
vibook.opt.plugins = { links = true }
```

---

## 4. Кеймапы — `vibook.keymap(mode, lhs, rhs)`

Маппинги normal-режима (`"n"`). Правая часть — это либо `:команда`, либо
последовательность клавиш, подаваемая редактору. Встроенные клавиши имеют
приоритет.

```lua
vibook.keymap("n", "<C-s>", ":w")   -- Ctrl-s сохраняет
vibook.keymap("n", "H", "0")        -- H в начало строки
vibook.keymap("n", "Y", "y$")       -- Y копирует до конца строки
```

Поддерживаются одиночные клавиши и один аккорд `<C-x>`. Многоклавишные
последовательности вроде `gh` пока не применяются.

---

## 5. Меню, команды, события

### Пункты меню — `vibook.menu_item(key, label, rhs)`

```lua
vibook.menu_item("g", "Open scratch", ":new scratch")
```

### Команды — `vibook.command(name, fn)`

Вызываются как `:name`. Внутри читайте и переписывайте активный буфер.

```lua
vibook.command("upper", function()
  vibook.buf.set_text(vibook.buf.get_text():upper())
end)

vibook.command("wc", function()
  local n = select(2, vibook.buf.get_text():gsub("%S+", ""))
  vibook.vault.create("wordcount-" .. n)
end)
```

### События — `vibook.on(event, fn)`

| Событие | Аргумент | Когда срабатывает |
|---------|----------|-------------------|
| `note_open` | — | заметка только что открыта |
| `note_save` | — | заметка только что записана |
| `follow_link` | `target` (string) | пользователь переходит по `[[ссылке]]` |

```lua
vibook.on("note_open", function()
  -- например, нормализовать буфер при открытии
end)

vibook.on("follow_link", function(target)
  -- решить, как открыть цель ссылки (см. §6 и §9)
end)
```

Все обработчики одного события выполняются в порядке регистрации.

---

## 6. Открытие заметок и файлов

Эти функции выражают *намерение*; vi-book выполняет действие после возврата
из обработчика.

- `vibook.open(target)` — открыть `target` внутри vi-book (путь заметки, напр.
  `"ideas.md"` или вложенный `"proj/spec.md"`).
- `vibook.open_external(target)` — открыть `target` во внешнем редакторе
  (`vibook.opt.code_editor`, иначе `$EDITOR`). vi-book находит файл в волте
  (по относительному пути, иначе по имени) и приостанавливает TUI на время
  работы редактора.
- `vibook.vault.create(name)` — создать новую заметку `name`.
- `vibook.vault.open(name)` — псевдоним `vibook.open` для симметрии с
  остальными `vault.*`.

---

## 7. Чтение состояния

### Буфер — `vibook.buf`

- `vibook.buf.get_text()` → активный буфер строкой.
- `vibook.buf.set_text(s)` → заменить активный буфер.
- `vibook.buf.get_cursor()` → символьный индекс курсора.

### Волт — `vibook.vault`

- `vibook.vault.notes()` → список путей заметок в волте.
- `vibook.vault.tags()` → список `#тегов` по всему волту.
- `vibook.vault.backlinks(path)` → заметки, ссылающиеся на `path`.

*Волт* — это активная папка, с которой открыт vi-book; поиск тегов и ссылок
ограничен ею.

---

## 8. Палитра подсветки — `vibook.color(name, spec)`

Код-блоки подсвечиваются вкомпилированными грамматиками tree-sitter
(rust, python, javascript, typescript, json, bash, go, c, toml, html, yaml и
популярные алиасы). Перекрашивайте любой токен — или любой markdown-стиль — из
конфига.

Поля `spec`: `fg` (цвет), `bold`, `italic`, `underline`, `dim`. `fg`
принимает имена терминальных цветов (`"red"`, `"magenta"`, `"cyan"`, …) и hex
(`"#82aaff"`).

```lua
vibook.color("keyword",  { fg = "magenta", bold = true })
vibook.color("string",   { fg = "green" })
vibook.color("number",   { fg = "yellow" })
vibook.color("comment",  { fg = "#6a737d", italic = true })
vibook.color("function", { fg = "blue" })
vibook.color("type",     { fg = "cyan" })

-- markdown-стили тоже настраиваются:
vibook.color("heading1", { fg = "magenta", bold = true })
vibook.color("link",     { fg = "blue", underline = true })
```

**Имена токенов кода:** `keyword`, `string`, `number`, `comment`, `function`,
`type`, `constant`, `variable`, `operator`, `punctuation`, `property`,
`attribute`.

**Имена markdown-стилей:** `text`, `heading1`, `heading2`, `heading3`, `bold`,
`italic`, `codespan`, `codeblock`, `frame`, `link`, `hashtag`, `quote`,
`frontmatter`.

> vi-book — терминальное приложение: оно не выбирает *шрифт*. Для лигатур и
> глифов поставьте в терминале Nerd Font (установщик предлагает
> JetBrainsMono Nerd Font).

---

## 9. Разбор примера — плагин `links`

Это дефолтный плагин, поставляемый в бинаре (`assets/plugins/links.lua`). Он
решает, как открыть цель `[[ссылки]]`: текстовые заметки — в vi-book, файлы с
кодом — в вашем редакторе, голые имена — отдаёт собственному резолверу vi-book.

```lua
local text_ext = {
  md = true, markdown = true, txt = true, text = true, org = true, rst = true,
}

local function disabled()
  return vibook.opt.plugins ~= nil and vibook.opt.plugins.links == false
end

local function extension(target)
  return target:match("%.([%w]+)$")
end

vibook.on("follow_link", function(target)
  if disabled() then
    return
  end
  local ext = extension(target)
  if ext == nil then
    return                      -- [[note]] → vi-book резолвит алиасы/якоря
  end
  if text_ext[ext:lower()] then
    vibook.open(target)         -- [[notes.md]] → открыть в vi-book
  else
    vibook.open_external(target) -- [[main.rs]] → внешний редактор
  end
end)
```

Как это работает:

1. `vibook.on("follow_link", …)` регистрирует обработчик, получающий строку
   цели ссылки (всё внутри `[[ ]]`, без `#якоря`/`|алиаса`).
2. `disabled()` уважает `vibook.opt.plugins.links = false`, чтобы плагин можно
   было выключить, не удаляя.
3. Цель **без** расширения завершает обработчик рано. Намерения открыть не
   создано, поэтому vi-book откатывается к встроенному резолверу (коллизии
   имён, алиасы, якоря).
4. **Текстовое** расширение вызывает `vibook.open` — vi-book открывает внутри.
5. Любое другое расширение вызывает `vibook.open_external` — vi-book находит
   файл в волте и передаёт его вашему редактору.

### Свой вариант

Отключите дефолтный и зарегистрируйте свою маршрутизацию — например, открывать
картинки просмотрщиком:

```lua
vibook.opt.plugins = { links = false }

vibook.on("follow_link", function(target)
  local ext = (target:match("%.([%w]+)$") or ""):lower()
  if ext == "png" or ext == "jpg" then
    vibook.open_external(target)   -- code_editor указывает на просмотрщик
  elseif ext == "md" or ext == "" then
    vibook.open(target)
  else
    vibook.open_external(target)
  end
end)
```

---

## 10. Сборка и тесты

```
cargo build --release   # один статический бинарь target/release/vi-book
./install.sh            # сборка + установка + конфиг + загрузка Nerd Font
cargo test              # каталог tests/ — иммутабельный контракт
```

Добавляя новую возможность в рантайм, покрывайте её тестом в `tests/`
(загрузите скрипт в `PluginHost`, затем проверьте наблюдаемый эффект),
по образцу существующих файлов `lua_*`.
