import random

WORDS = "alpha beta gamma delta note link vault idea draft review plan task".split()
TOPICS = "rust lua terminal editor parser render index cache layout theme".split()
SECTIONS = ("knowledge", "projects", "areas", "inbox", "daily")
WIKI_NOTES = 1500
HEAVY_NOTES = 60
COMMON = ("README", "index", "todo")


def note(rng, index, count):
    links = " ".join(f"[[note-{rng.randrange(count):05}]]" for _ in range(3))
    tags = " ".join(f"#tag-{rng.randrange(50)}" for _ in range(2))
    prose = "\n\n".join(" ".join(rng.choices(WORDS, k=60)) for _ in range(3))
    items = "\n".join(f"- {' '.join(rng.choices(WORDS, k=6))}" for _ in range(4))
    return (
        f"# Note {index}\n\n{tags}\n\n{prose}\n\n## Links\n\n{links}\n\n"
        f"{items}\n\n```rust\nfn note_{index}() -> usize {{ {index} }}\n```\n"
    )


def vault(base, count):
    path = base / f"vault-{count}"
    path.mkdir()
    rng = random.Random(count)
    for index in range(count):
        (path / f"note-{index:05}.md").write_text(note(rng, index, count))
    return path


def sentence(rng, words):
    return " ".join(rng.choices(WORDS + TOPICS, k=words)).capitalize() + "."


def wiki_link(rng, names):
    target = rng.choice(names)
    shape = rng.randrange(4)
    if shape == 0:
        return f"[[{target}|{rng.choice(WORDS)}]]"
    if shape == 1:
        return f"[[{target}#{rng.choice(TOPICS).capitalize()}]]"
    return f"[[{target}]]"


def frontmatter(rng, index):
    tags = ", ".join(rng.sample(TOPICS, 3))
    return f"---\ntags: [{tags}]\ncreated: 2026-{index % 12 + 1:02}-{index % 28 + 1:02}\n---\n\n"


def code_block(rng, index):
    lang = rng.choice(("rust", "lua", "sh", "toml"))
    body = "\n".join(f"let {rng.choice(WORDS)}_{index} = {rng.randrange(1000)};" for _ in range(2))
    return f"```{lang}\n{body}\n```\n"


def plain_note(rng, index, names):
    parts = [frontmatter(rng, index), f"# {rng.choice(TOPICS).capitalize()} {index}\n\n"]
    for heading in rng.sample(TOPICS, 5):
        parts.append(f"## {heading.capitalize()}\n\n")
        prose = " ".join(sentence(rng, 14) for _ in range(5))
        links = " ".join(wiki_link(rng, names) for _ in range(3))
        parts.append(f"{prose} {links} #{rng.choice(TOPICS)}\n\n")
        parts.append("\n".join(f"- {sentence(rng, 8)}" for _ in range(4)) + "\n\n")
        if rng.random() < 0.4:
            parts.append(code_block(rng, index) + "\n")
    return "".join(parts)


def heavy_note(rng, index, names):
    parts = [frontmatter(rng, index), f"# Dev log {index}\n\n"]
    for step in range(rng.randrange(400, 1201)):
        parts.append(f"{sentence(rng, 6)} {wiki_link(rng, names) if step % 9 == 0 else ''}\n\n")
        parts.append(code_block(rng, step) + "\n")
    return "".join(parts)


def wiki_paths(rng):
    paths = []
    for index in range(WIKI_NOTES - HEAVY_NOTES):
        section = SECTIONS[index % len(SECTIONS)]
        topic = TOPICS[index % len(TOPICS)]
        nested = f"{section}/{topic}/{rng.choice(WORDS)}" if index % 3 == 0 else f"{section}/{topic}"
        common = COMMON[index // 37 % len(COMMON)]
        stem = common if index % 37 == 0 else f"{topic}-{index:04}"
        paths.append(f"{nested}/{stem}.md")
    paths += [
        f"dev-log/{TOPICS[i % len(TOPICS)]}/26-10-{i:02}-log.md" for i in range(HEAVY_NOTES)
    ]
    return sorted(set(paths))


def wiki(base):
    path = base / "wiki-like"
    rng = random.Random(1444)
    paths = wiki_paths(rng)
    names = [rel.rsplit("/", 1)[-1].removesuffix(".md") for rel in paths]
    for index, rel in enumerate(paths):
        target = path / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        build = heavy_note if rel.startswith("dev-log/") else plain_note
        target.write_text(build(rng, index, names))
    return path, len(paths)
