#!/usr/bin/env python3
"""Typeset docs/USERGUIDE.md and materialize its named teaching examples.

Requires reportlab. Prefer New Century Schoolbook, then TeX Gyre Schola.
Run from any directory: python3 scripts/build_userguide.py
"""
from __future__ import annotations

import argparse
import html
import math
from pathlib import Path
import re
import shutil
import subprocess
import textwrap
import xml.etree.ElementTree as ET

from reportlab.lib import colors
from reportlab.lib.styles import ParagraphStyle
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate, Frame, PageTemplate, Paragraph, Spacer, PageBreak,
    Preformatted, Table, TableStyle, Flowable, KeepTogether,
)
from reportlab.platypus.tableofcontents import TableOfContents

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'docs/USERGUIDE.md'
INK = colors.HexColor('#163d3a')
GOLD = colors.HexColor('#b88a36')
PALE = colors.HexColor('#f1f5f3')
WIDTH, HEIGHT = 540, 720
MARGIN = 51
BODY_WIDTH = WIDTH - 2 * MARGIN


def register_fonts():
    exact = ROOT / 'assets/fonts'
    exact_names = [('Book', 'NewCenturySchoolbook-Roman.ttf'),
                   ('Book-Bold', 'NewCenturySchoolbook-Bold.ttf'),
                   ('Book-Italic', 'NewCenturySchoolbook-Italic.ttf'),
                   ('Book-BoldItalic', 'NewCenturySchoolbook-BoldItalic.ttf')]
    if all((exact / filename).exists() for _, filename in exact_names):
        for name, filename in exact_names:
            pdfmetrics.registerFont(TTFont(name, str(exact / filename)))
        pdfmetrics.registerFontFamily('Book', normal='Book', bold='Book-Bold',
                                      italic='Book-Italic', boldItalic='Book-BoldItalic')
        return 'Book', 'Book-Bold', 'Book-Italic'

    schola_names = [('Book', 'qcsr'), ('Book-Bold', 'qcsb'),
                    ('Book-Italic', 'qcsri'), ('Book-BoldItalic', 'qcsbi')]
    schola_paths = []
    if shutil.which('kpsewhich'):
        for name, stem in schola_names:
            paths = []
            for suffix in ('afm', 'pfb'):
                found = subprocess.run(['kpsewhich', f'{stem}.{suffix}'],
                                       capture_output=True, text=True, check=False)
                paths.append(Path(found.stdout.strip()))
            schola_paths.append((name, *paths))
    if len(schola_paths) == len(schola_names) and all(
        afm.is_file() and pfb.is_file() for _, afm, pfb in schola_paths
    ):
        for name, afm, pfb in schola_paths:
            face = pdfmetrics.EmbeddedType1Face(str(afm), str(pfb))
            pdfmetrics.registerTypeFace(face)
            pdfmetrics.registerFont(pdfmetrics.Font(name, face.name, 'WinAnsiEncoding'))
        pdfmetrics.registerFontFamily('Book', normal='Book', bold='Book-Bold',
                                      italic='Book-Italic', boldItalic='Book-BoldItalic')
        return 'Book', 'Book-Bold', 'Book-Italic'

    folder = Path('/System/Library/Fonts/Supplemental')
    names = [('Book', 'Georgia.ttf'), ('Book-Bold', 'Georgia Bold.ttf'),
             ('Book-Italic', 'Georgia Italic.ttf'),
             ('Book-BoldItalic', 'Georgia Bold Italic.ttf')]
    if all((folder / name).exists() for _, name in names):
        for name, file in names:
            pdfmetrics.registerFont(TTFont(name, str(folder / file)))
        pdfmetrics.registerFontFamily('Book', normal='Book', bold='Book-Bold',
                                      italic='Book-Italic', boldItalic='Book-BoldItalic')
        return 'Book', 'Book-Bold', 'Book-Italic'
    return 'Times-Roman', 'Times-Bold', 'Times-Italic'


ROMAN, BOLD, ITALIC = register_fonts()
STYLES = {
    'body': ParagraphStyle('body', fontName=ROMAN, fontSize=10.4, leading=15.2,
                           spaceAfter=8, textColor=colors.HexColor('#253633')),
    'chapter': ParagraphStyle('chapter', fontName=BOLD, fontSize=24, leading=29,
                              textColor=INK, spaceBefore=12, spaceAfter=20),
    'section': ParagraphStyle('section', fontName=BOLD, fontSize=13.5, leading=18,
                              textColor=INK, spaceBefore=12, spaceAfter=7, keepWithNext=True),
    'subsection': ParagraphStyle('subsection', fontName=BOLD, fontSize=11, leading=15,
                                 textColor=INK, spaceBefore=10, spaceAfter=5, keepWithNext=True),
    'code': ParagraphStyle('code', fontName='Courier', fontSize=7.5, leading=10.2,
                           backColor=PALE, borderPadding=8, spaceBefore=5, spaceAfter=12),
    'caption': ParagraphStyle('caption', fontName=ITALIC, fontSize=8.6, leading=12,
                              textColor=INK, spaceBefore=5, spaceAfter=10),
    'table': ParagraphStyle('table', fontName=ROMAN, fontSize=8.5, leading=11.8),
    'label': ParagraphStyle('label', fontName=BOLD, fontSize=9, leading=12,
                            textColor=GOLD, spaceAfter=8),
}


def inline(text):
    value = html.escape(text)
    value = re.sub(r'\[([^]]+)\]\((https?://[^)]+)\)', r'<link href="\2">\1</link>', value)
    value = re.sub(r'`([^`]+)`', r'<font name="Courier" size="8.8">\1</font>', value)
    value = re.sub(r'\*\*([^*]+)\*\*', r'<b>\1</b>', value)
    return value


def materialize_examples(text):
    names = []
    for match in re.finditer(r'```(?:yaml|json) file=(examples/userguide/[\w.-]+)\n(.*?)\n```', text, re.S):
        path = ROOT / match[1]
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(match[2] + '\n')
        names.append(match[1])
    return names


def verify_atlas(text):
    pattern = r'### `(examples/[^`]+\.yaml)`\nSource file:.*?\n\n```yaml\n(.*?)\n```'
    entries = re.findall(pattern, text, re.S)
    expected = {p.relative_to(ROOT).as_posix() for p in (ROOT / 'examples').rglob('*.yaml')
                if 'userguide' not in p.parts}
    found = {path for path, _ in entries}
    if found != expected or len(entries) != len(found):
        raise ValueError(f'Atlas inventory differs from examples/: missing={expected-found}, extra={found-expected}')

    def without_trailing_space(value):
        return '\n'.join(line.rstrip() for line in value.splitlines()).rstrip()
    for path, contents in entries:
        if without_trailing_space((ROOT / path).read_text()) != without_trailing_space(contents):
            raise ValueError(f'Atlas entry differs from source: {path}')
    return len(entries)


class Diagram(Flowable):
    def __init__(self, kind):
        super().__init__()
        self.kind = kind
        self.width = BODY_WIDTH
        self.height = 175

    def draw(self):
        c = self.canv
        c.setFillColor(PALE)
        c.roundRect(0, 0, self.width, self.height, 5, fill=1, stroke=0)
        c.setStrokeColor(INK)
        c.setFillColor(INK)
        c.setFont('Helvetica', 9)
        if self.kind == 'signal-flow':
            for i, (top, bottom) in enumerate([('SOURCE', 'WAV in memory'), ('CLOUD', 'windowed grains'), ('LAYOUT', 'speaker gains'), ('OUTPUT', 'WAV or device')]):
                x = 12 + 107 * i
                c.setStrokeColor(GOLD)
                c.roundRect(x, 65, 94, 58, 3, fill=0, stroke=1)
                c.setFont('Helvetica-Bold', 9)
                c.drawCentredString(x + 47, 100, top)
                c.setFont('Helvetica', 8)
                c.drawCentredString(x + 47, 81, bottom)
                if i < 3:
                    c.line(x + 94, 94, x + 105, 94)
            c.drawCentredString(self.width / 2, 30, 'Signals and control fields shape each cloud before grains are mixed.')
        elif self.kind in ('overlap', 'windows'):
            x0, y0, w, h = 35, 35, 365, 110
            c.line(x0, y0, x0 + w, y0)
            c.line(x0, y0, x0, y0 + h)
            for j in range(3):
                c.setStrokeColor([INK, GOLD, colors.HexColor('#78948b')][j])
                c.setLineWidth(1.6)
                path = c.beginPath()
                for i in range(161):
                    t = i / 160
                    if self.kind == 'windows':
                        v = [1.0, 0.5 - 0.5 * math.cos(2 * math.pi * t),
                             0.35875 - 0.48829 * math.cos(2 * math.pi * t) + 0.14128 * math.cos(4 * math.pi * t) - 0.01168 * math.cos(6 * math.pi * t)][j]
                        x = x0 + w * t
                    else:
                        v = 0.5 - 0.5 * math.cos(2 * math.pi * t)
                        x = x0 + j * 92 + t * 180
                    y = y0 + 85 * v
                    (path.moveTo if i == 0 else path.lineTo)(x, y)
                c.drawPath(path)
            c.setFillColor(INK)
            c.setFont('Helvetica', 8)
            c.drawString(35, 15, 'time within a grain' if self.kind == 'windows' else 'time: overlapping envelopes sum at the output')
        elif self.kind == 'scan':
            c.line(50, 35, 398, 35)
            c.line(50, 35, 50, 147)
            c.setStrokeColor(GOLD)
            c.setLineWidth(2)
            c.line(50, 35, 360, 128)
            c.line(360, 128, 398, 128)
            c.drawString(64, 140, 'source position (seconds)')
            c.drawString(278, 15, 'output time (seconds)')
            c.drawString(36, 31, '0')
            c.drawString(36, 124, '2')
            c.drawString(357, 23, '8')
            c.drawString(154, 63, 'two seconds of source positions')
            c.drawString(154, 50, 'spread across eight seconds of output')
        elif self.kind == 'layout':
            cx, cy, r = 215, 84, 56
            c.circle(cx, cy, r, stroke=1, fill=0)
            for label, angle in [('FL', -45), ('FR', 45), ('RL', -135), ('RR', 135)]:
                a = math.radians(angle)
                x, y = cx + r * math.sin(a), cy + r * math.cos(a)
                c.setFillColor(GOLD)
                c.circle(x, y, 5, fill=1, stroke=0)
                c.setFillColor(INK)
                c.drawCentredString(x, y + (12 if y > cy else -17), label)
            c.circle(cx, cy, 3, fill=1, stroke=0)
            c.drawString(303, 84, '+x right')
            c.drawCentredString(cx, 159, '+y forward')
            c.drawString(24, 10, 'Azimuth is measured from forward; positive angles turn right.')


class RepositoryLogo(Flowable):
    """Draw the repository SVG's geometry without a raster conversion dependency."""

    def __init__(self):
        super().__init__()
        self.width = BODY_WIDTH
        self.height = BODY_WIDTH * 360 / 1400

    def draw(self):
        root = ET.parse(ROOT / 'assets/logo.svg').getroot()
        scale = self.width / float(root.attrib['width'])
        c = self.canv
        c.saveState()
        c.scale(scale, scale)
        c.setFillColor(colors.white)
        c.rect(0, 0, 1400, 360, fill=1, stroke=0)
        mark = root.find('.//{*}g[@transform="translate(380,180)"]')
        assert mark is not None
        c.setStrokeColor(colors.Color(0, 0, 0, alpha=0.28))
        c.setLineWidth(3)
        for orbit in mark.findall('.//{*}ellipse'):
            rx, ry = float(orbit.attrib['rx']), float(orbit.attrib['ry'])
            c.ellipse(380-rx, 180-ry, 380+rx, 180+ry, fill=0, stroke=1)
        c.setFillColor(colors.black)
        for particle in mark.findall('.//{*}circle'):
            x = 380 + float(particle.attrib['cx'])
            y = 180 - float(particle.attrib['cy'])
            c.circle(x, y, float(particle.attrib['r']), fill=1, stroke=0)
        wordmark = root.find('.//{*}text')
        assert wordmark is not None
        c.setFont(ROMAN, 110)
        c.drawString(610, 150, ''.join(wordmark.itertext()).strip())
        c.restoreState()


CAPTIONS = {
    'signal-flow': 'Figure 1. The granular signal path. A patch coordinates source selection, grain shape, and output geometry.',
    'overlap': 'Figure 2. Three overlapping Hann envelopes. This schematic shows overlap, not a capture of scheduler timing.',
    'scan': 'Figure 3. The teaching scan curve reaches two seconds of source position at eight seconds of output, then clamps.',
    'layout': 'Figure 4. The four-channel teaching layout viewed from above. Channel order is FL, FR, RL, RR.',
    'windows': 'Figure 5. Rectangular (green), Hann (gold), and Blackman-Harris (gray-green) windows at equal peak height.',
}


class Book(BaseDocTemplate):
    def __init__(self, path):
        super().__init__(str(path), pagesize=(WIDTH, HEIGHT),
                         leftMargin=MARGIN, rightMargin=MARGIN,
                         topMargin=49, bottomMargin=49,
                         title='Particelle User Guide', author='Colby Leider, PhD',
                         subject='Granular sound design, spatial composition, and expressive control')
        frame = Frame(MARGIN, 49, BODY_WIDTH, HEIGHT - 98, leftPadding=0,
                      rightPadding=0, topPadding=0, bottomPadding=0)
        self.addPageTemplates(PageTemplate(id='Book', frames=[frame], onPage=self.page_furniture))

    def beforeDocument(self):
        self.chapter = 'Particelle User Guide'
        self.serial = 0

    def page_furniture(self, c, doc):
        if doc.page == 1:
            return
        c.saveState()
        c.setStrokeColor(GOLD)
        c.setLineWidth(0.5)
        c.line(MARGIN, HEIGHT - 32, WIDTH - MARGIN, HEIGHT - 32)
        c.setFillColor(INK)
        c.setFont(ROMAN, 8)
        c.drawString(MARGIN, HEIGHT - 25, 'PARTICELLE  /  USER GUIDE')
        c.drawRightString(WIDTH - MARGIN, 28, str(doc.page))
        c.drawString(MARGIN, 28, 'Source-tree edition  /  October 2026')
        c.restoreState()

    def afterFlowable(self, flowable):
        if isinstance(flowable, Paragraph) and getattr(flowable, 'toc_level', None) is not None:
            level = flowable.toc_level
            title = flowable.getPlainText()
            key = f'section-{self.serial}'
            self.serial += 1
            self.canv.bookmarkPage(key)
            self.canv.addOutlineEntry(title, key, level=level, closed=level > 0)
            self.notify('TOCEntry', (level, title, self.page, key))


def table(lines):
    rows = []
    for line in lines:
        cells = [s.strip() for s in line.strip().strip('|').split('|')]
        if all(re.fullmatch(r':?-+:?', cell.replace(' ', '')) for cell in cells):
            continue
        rows.append([Paragraph(inline(cell), STYLES['table']) for cell in cells])
    n = len(rows[0])
    widths = [BODY_WIDTH * v for v in ([0.29, 0.71] if n == 2 else [0.24, 0.32, 0.44])]
    result = Table(rows, colWidths=widths, repeatRows=1, hAlign='LEFT')
    result.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#dae6df')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, PALE]),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
        ('LEFTPADDING', (0, 0), (-1, -1), 7),
        ('RIGHTPADDING', (0, 0), (-1, -1), 7),
        ('TOPPADDING', (0, 0), (-1, -1), 7),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 7),
        ('LINEBELOW', (0, 0), (-1, 0), 0.6, GOLD),
    ]))
    return result


def build(text, output):
    story = []
    subtitle_style = ParagraphStyle('subtitle', fontName=ROMAN, fontSize=19, leading=26, textColor=INK)
    story += [Spacer(1, 45), RepositoryLogo(), Spacer(1, 10),
              Paragraph('User Guide', subtitle_style), Spacer(1, 35), Diagram('overlap'),
              Spacer(1, 27), Paragraph('Granular sound design, spatial composition,<br/>and expressive control', subtitle_style),
              Spacer(1, 26), Paragraph('Practical workflows / Illustrated concepts<br/>Listening exercises / Command and patch reference', STYLES['body']),
              Spacer(1, 12), Paragraph('Colby Leider, PhD', STYLES['body']),
              Spacer(1, 10), Paragraph('SOURCE-TREE EDITION  /  OCTOBER 10, 2026', STYLES['label']), PageBreak()]
    toc = TableOfContents()
    toc.levelStyles = [ParagraphStyle('toc', fontName=ROMAN, fontSize=9.0, leading=10.5, spaceBefore=0),
                       ParagraphStyle('tocsub', fontName=ROMAN, fontSize=8.2, leading=10.5, leftIndent=14)]
    story += [Paragraph('Contents', STYLES['chapter']), toc, PageBreak()]
    lines = text.splitlines()
    i = 1
    while i < len(lines):
        line = lines[i].strip()
        if not line:
            i += 1
            continue
        if line.startswith('!['):
            i += 1
            continue
        if line.startswith('# '):
            story.append(PageBreak())
            p = Paragraph(inline(line[2:]), STYLES['chapter'])
            p.toc_level = 0
            story.append(p)
        elif line.startswith('## '):
            if line[3:] == 'How to use this book':
                story.append(PageBreak())
            p = Paragraph(inline(line[3:]), STYLES['section'])
            # Front matter is listed in the TOC; chapter subheadings stay in text.
            if line[3:] in ('Preface', 'How to use this book'):
                p.toc_level = 0
            story.append(p)
        elif line.startswith('### '):
            story.append(Paragraph(inline(line[4:]), STYLES['subsection']))
        elif line.startswith('```'):
            info = line[3:]
            code = []
            i += 1
            while i < len(lines) and not lines[i].startswith('```'):
                code.extend(textwrap.wrap(lines[i], 88, replace_whitespace=False,
                                          drop_whitespace=False) or [''])
                i += 1
            block = []
            if 'file=' in info:
                block.append(Paragraph(html.escape(info.split('file=')[1]), STYLES['caption']))
            for start in range(0, len(code), 38):
                page = code[start:start + 38]
                if start:
                    block = []
                block.append(Preformatted('\n'.join(page), STYLES['code']))
                story.append(KeepTogether(block))
        elif line.startswith('|'):
            block = []
            while i < len(lines) and lines[i].strip().startswith('|'):
                block.append(lines[i])
                i += 1
            story.extend([table(block), Spacer(1, 10)])
            continue
        elif line.startswith('[Figure: '):
            kind = line[9:-1]
            story.append(KeepTogether([Diagram(kind), Paragraph(CAPTIONS[kind], STYLES['caption'])]))
        elif line.startswith('- '):
            story.append(Paragraph(inline(line[2:]), STYLES['body'], bulletText='-'))
        else:
            paragraph = [line]
            while i + 1 < len(lines) and lines[i + 1].strip() and not lines[i + 1].startswith(('#', '|', '```', '[Figure:', '- ')):
                i += 1
                paragraph.append(lines[i])
            story.append(Paragraph(inline(' '.join(paragraph)), STYLES['body']))
        i += 1
    output.parent.mkdir(parents=True, exist_ok=True)
    Book(output).multiBuild(story)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'output/pdf/Particelle_User_Guide.pdf')
    args = parser.parse_args()
    text = SOURCE.read_text()
    atlas_count = verify_atlas(text)
    examples = materialize_examples(text)
    build(text, args.output)
    print(f'Built {args.output}; materialized {len(examples)} teaching files; verified {atlas_count} atlas entries.')


if __name__ == '__main__':
    main()
