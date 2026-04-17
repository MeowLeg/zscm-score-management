#!/usr/bin/env python3
"""Unified Newspaper Scraper & Vectorizer for 舟山日报 & 舟山晚报"""

import sqlite3
import requests
from bs4 import BeautifulSoup
import re
import json
import logging
from calendar import monthrange
from pathlib import Path
from typing import List, Optional, Dict

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class NewspaperScraper:
    NEWSPAPERS = {
        "zsrb": {
            "name": "舟山日报",
            "base_url": "https://epaper.wifizs.cn/zsrb",
            "db_path": "newspaper_zsrb_data.db",
        },
        "zswb": {
            "name": "舟山晚报",
            "base_url": "https://epaper.wifizs.cn/zswb",
            "db_path": "newspaper_data.db",
        },
    }

    def __init__(self, newspaper: str = "zsrb", db_path: str = None):
        if newspaper not in self.NEWSPAPERS:
            raise ValueError(
                f"Unknown newspaper: {newspaper}. Available: {list(self.NEWSPAPERS.keys())}"
            )

        config = self.NEWSPAPERS[newspaper]
        self.name = config["name"]
        self.BASE_URL = config["base_url"]
        self.db_path = db_path or config["db_path"]

        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            }
        )

    def _create_database_tables(self):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT, publication_date DATE NOT NULL,
            issue_number INTEGER, edition TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)""")
        cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT, issue_id INTEGER NOT NULL,
            page_number INTEGER NOT NULL, page_info TEXT, url TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (issue_id) REFERENCES newspaper_issues(id) ON DELETE CASCADE)""")
        cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT, page_id INTEGER NOT NULL,
            title TEXT NOT NULL, content TEXT, article_url TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (page_id) REFERENCES newspaper_pages(id) ON DELETE CASCADE,
            UNIQUE(article_url))""")
        conn.commit()
        conn.close()
        logger.info(f"[{self.name}] Database tables ready")

    def _create_or_get_issue(self, publication_date: str) -> int:
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(
            "SELECT id FROM newspaper_issues WHERE publication_date = ?",
            (publication_date,),
        )
        result = cursor.fetchone()
        if result:
            issue_id = result[0]
        else:
            cursor.execute(
                "INSERT INTO newspaper_issues (publication_date, issue_number, edition) VALUES (?, 1, ?)",
                (publication_date, f"{self.name} Edition"),
            )
            issue_id = cursor.lastrowid
        conn.commit()
        conn.close()
        return issue_id

    def _create_or_get_page(
        self, issue_id: int, page_number: int, page_info: str, url: str
    ) -> int:
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(
            "SELECT id FROM newspaper_pages WHERE issue_id = ? AND page_number = ?",
            (issue_id, page_number),
        )
        result = cursor.fetchone()
        if result:
            page_id = result[0]
        else:
            cursor.execute(
                "INSERT INTO newspaper_pages (issue_id, page_number, page_info, url) VALUES (?, ?, ?, ?)",
                (issue_id, page_number, page_info, url),
            )
            page_id = cursor.lastrowid
        conn.commit()
        conn.close()
        return page_id

    def _extract_page_info(self, soup) -> str:
        header = soup.find("div", class_=re.compile(r"header|top|masthead", re.I))
        if header:
            page_info_text = header.get_text(strip=True)
            page_match = re.search(r"(\d+版)[:：]?(.*)", page_info_text)
            if page_match:
                return f"{page_match.group(1)}{page_match.group(2)}"
        page_title = soup.find("h2") or soup.find("h1")
        return page_title.get_text(strip=True) if page_title else "Unknown Page"

    def _extract_articles(self, soup, page_id: int, url_date: str):
        navigation_patterns = [
            r"上一版",
            r"下一版",
            r"上一期",
            r"下一期",
            r"数字报首页",
            r"首页",
            r"广告",
            r"快速便捷",
            r"导读",
        ]
        navigation_keywords = {
            "< 上一期",
            "> 下一期",
            "数字报首页",
            "首页",
            "广告",
            "快速便捷",
        }

        content_div = soup.find(
            "div", class_=re.compile(r"content|list|articles|news", re.I)
        )
        if not content_div:
            content_div = soup.find("div") or soup.find("ul")

        article_links = (
            content_div.find_all("a", href=re.compile(r"content_"))
            if content_div
            else []
        )
        if not article_links:
            article_links = soup.find_all("a", href=re.compile(r"\.html"))

        for link in article_links:
            article_url = link.get("href")
            title = link.get_text(strip=True)
            if not title:
                continue

            is_navigation = False
            for pattern in navigation_patterns:
                if re.search(pattern, title):
                    is_navigation = True
                    break

            if title in navigation_keywords:
                is_navigation = True

            if re.match(r"^\d+版[:：].*$", title) or re.match(r"^\d+$", title.strip()):
                is_navigation = True

            if is_navigation:
                continue

            article_url = self._construct_article_url(article_url, url_date)
            content = self._extract_article_content(article_url)
            self._create_article(page_id, title, content, article_url)

    def _construct_article_url(self, article_url: str, url_date: str) -> Optional[str]:
        if not article_url:
            return None
        if article_url.startswith("http"):
            return article_url
        article_url = article_url.lstrip("./")
        if not article_url.startswith("/"):
            return f"{self.BASE_URL}/{url_date}/{article_url}"
        return f"{self.BASE_URL}{article_url}"

    def _extract_article_content(self, article_url: str) -> str:
        if not article_url:
            return ""
        try:
            response = self.session.get(article_url, timeout=10)
            response.raise_for_status()
            soup = BeautifulSoup(response.content, "html.parser")
            content_parts = []
            for div in soup.find_all(
                "div", class_=re.compile(r"content|article|body|text", re.I)
            ):
                for p in div.find_all("p"):
                    p_text = p.get_text(strip=True)
                    if p_text and len(p_text) > 5:
                        content_parts.append(p_text)
            if content_parts:
                return "\n".join(content_parts)
            body = soup.find("body")
            if body:
                text = body.get_text(separator="\n", strip=True)
                lines = [line.strip() for line in text.split("\n") if line.strip()]
                return "\n".join(lines)
        except Exception as e:
            logger.warning(f"Could not fetch content from {article_url}: {e}")
        return ""

    def _create_article(self, page_id: int, title: str, content: str, article_url: str):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(
            "SELECT id FROM newspaper_articles WHERE article_url = ?",
            (article_url,),
        )
        if cursor.fetchone():
            logger.debug(f"Article already exists, skipping: {article_url}")
            conn.close()
            return
        cursor.execute(
            "INSERT INTO newspaper_articles (page_id, title, content, article_url) VALUES (?, ?, ?, ?)",
            (page_id, title, content, article_url),
        )
        conn.commit()
        conn.close()

    def _scrape_single_date(
        self, publication_date: str, url_date: str
    ) -> Optional[int]:
        try:
            test_url = f"{self.BASE_URL}/{url_date}/node_1.html"
            response = self.session.get(test_url, timeout=10)
            if response.status_code != 200:
                return None

            issue_id = self._create_or_get_issue(publication_date)
            for page_num in range(1, 9):
                page_url = f"{self.BASE_URL}/{url_date}/node_{page_num}.html"
                try:
                    response = self.session.get(page_url, timeout=10)
                    response.raise_for_status()
                    soup = BeautifulSoup(response.content, "html.parser")
                    page_number = page_num
                    page_info = self._extract_page_info(soup)
                    page_id = self._create_or_get_page(
                        issue_id, page_number, page_info, page_url
                    )
                    self._extract_articles(soup, page_id, url_date)
                except requests.exceptions.HTTPError:
                    break
                except Exception as e:
                    logger.warning(f"Error on page {page_num}: {e}")
            return issue_id
        except Exception as e:
            return None

    def scrape_year(self, year: int, start_month: int = 1, end_month: int = 12) -> int:
        self._create_database_tables()
        total_articles = 0
        for month in range(start_month, end_month + 1):
            days_in_month = monthrange(year, month)[1]
            month_articles = 0
            for day in range(1, days_in_month + 1):
                date_str = f"{year}-{month:02d}-{day:02d}"
                url_date = f"{year}-{month:02d}/{day:02d}"
                try:
                    issue_id = self._scrape_single_date(date_str, url_date)
                    if issue_id:
                        conn = sqlite3.connect(self.db_path)
                        cursor = conn.cursor()
                        cursor.execute(
                            "SELECT COUNT(*) FROM newspaper_articles WHERE page_id IN (SELECT id FROM newspaper_pages WHERE issue_id = ?)",
                            (issue_id,),
                        )
                        count = cursor.fetchone()[0]
                        conn.close()
                        month_articles += count
                        logger.info(f"[{self.name}] {date_str}: {count} articles")
                except Exception as e:
                    logger.warning(f"[{self.name}] Could not scrape {date_str}: {e}")
            total_articles += month_articles
            logger.info(f"[{self.name}] Month {month} done: {month_articles} articles")
        logger.info(f"[{self.name}] Year {year} done! Total: {total_articles} articles")
        return total_articles


class TitleVectorizer:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self.cursor = self.conn.cursor()
        self._init_table()
        self.vocab: Dict[str, int] = {}
        self._load_or_build_vocab()

    def _init_table(self):
        self.cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_title_vector (
            id INTEGER PRIMARY KEY AUTOINCREMENT, article_id INTEGER NOT NULL,
            title TEXT NOT NULL, vector BLOB NOT NULL, vector_dim INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(article_id))""")
        self.cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_vector_vocab (
            id INTEGER PRIMARY KEY AUTOINCREMENT, word TEXT NOT NULL UNIQUE, idx INTEGER NOT NULL UNIQUE)""")
        self.conn.commit()

    def _tokenize(self, text: str) -> List[str]:
        stop_words = {
            "的",
            "是",
            "在",
            "有",
            "和",
            "与",
            "及",
            "或",
            "但",
            "而",
            "也",
            "都",
            "就",
            "被",
            "把",
            "让",
            "给",
            "到",
            "从",
            "向",
            "对",
            "于",
            "等",
            "这",
            "那",
            "个",
            "些",
            "之",
            "为",
            "以",
            "因",
            "所",
            "可以",
            "能够",
        }
        text = re.sub(r"[^\w\u4e00-\u9fff]", " ", text)
        tokens = []
        n = len(text)
        i = 0
        while i < n:
            if text[i] == " ":
                i += 1
                continue
            if i + 1 < n:
                two_char = text[i : i + 2]
                if two_char in stop_words:
                    i += 2
                    continue
            single_char = text[i]
            if single_char not in stop_words:
                tokens.append(single_char)
            i += 1
        return tokens

    def _load_or_build_vocab(self):
        row = self.cursor.execute(
            "SELECT COUNT(*) as cnt FROM newspaper_vector_vocab"
        ).fetchone()
        if row["cnt"] > 0:
            rows = self.cursor.execute(
                "SELECT word, idx FROM newspaper_vector_vocab ORDER BY idx"
            ).fetchall()
            self.vocab = {r["word"]: r["idx"] for r in rows}
        else:
            self._build_vocab()

    def _build_vocab(self):
        logger.info("Building vocabulary...")
        rows = self.cursor.execute("SELECT title FROM newspaper_articles").fetchall()
        word_count: Dict[str, int] = {}
        for row in rows:
            tokens = self._tokenize(row["title"])
            for token in tokens:
                word_count[token] = word_count.get(token, 0) + 1
        sorted_words = sorted(word_count.items(), key=lambda x: -x[1])[:10000]
        self.vocab = {word: i for i, (word, _) in enumerate(sorted_words)}
        for word, idx in self.vocab.items():
            self.cursor.execute(
                "INSERT INTO newspaper_vector_vocab (word, idx) VALUES (?, ?)",
                (word, idx),
            )
        self.conn.commit()
        logger.info(f"Vocabulary built: {len(self.vocab)} words")

    def get_embedding(self, text: str) -> Optional[List[float]]:
        tokens = self._tokenize(text)
        dim = len(self.vocab)
        vector = [0.0] * dim
        token_count: Dict[str, int] = {}
        for token in tokens:
            token_count[token] = token_count.get(token, 0) + 1
        max_count = max(token_count.values()) if token_count else 1
        for token, count in token_count.items():
            if token in self.vocab:
                vector[self.vocab[token]] = count / max_count
        return vector

    def vectorize_all(self, force: bool = False):
        query = "SELECT id, title FROM newspaper_articles"
        if not force:
            query += " WHERE id NOT IN (SELECT article_id FROM newspaper_title_vector)"
        query += " ORDER BY id"
        rows = self.cursor.execute(query).fetchall()
        logger.info(f"Titles to vectorize: {len(rows)}")
        for i, row in enumerate(rows, 1):
            vector = self.get_embedding(row["title"])
            if vector:
                vector_blob = json.dumps(vector).encode("utf-8")
                self.cursor.execute(
                    """INSERT OR REPLACE INTO newspaper_title_vector
                    (article_id, title, vector, vector_dim, updated_at)
                    VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)""",
                    (row["id"], row["title"], vector_blob, len(vector)),
                )
                if i % 100 == 0:
                    self.conn.commit()
        self.conn.commit()
        logger.info(f"Vectorization done! Processed {len(rows)} titles")

    def close(self):
        self.conn.close()


class ContentVectorizer:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self.cursor = self.conn.cursor()
        self._init_table()
        self.vocab: Dict[str, int] = {}
        self._load_or_build_vocab()

    def _init_table(self):
        self.cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_content_vector (
            id INTEGER PRIMARY KEY AUTOINCREMENT, article_id INTEGER NOT NULL,
            content TEXT NOT NULL, vector BLOB NOT NULL, vector_dim INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(article_id))""")
        self.cursor.execute("""CREATE TABLE IF NOT EXISTS newspaper_content_vocab (
            id INTEGER PRIMARY KEY AUTOINCREMENT, word TEXT NOT NULL UNIQUE, idx INTEGER NOT NULL UNIQUE)""")
        self.conn.commit()

    def _tokenize(self, text: str) -> List[str]:
        stop_words = {
            "的",
            "是",
            "在",
            "有",
            "和",
            "与",
            "及",
            "或",
            "但",
            "而",
            "也",
            "都",
            "就",
            "被",
            "把",
            "让",
            "给",
            "到",
            "从",
            "向",
            "对",
            "于",
            "等",
            "这",
            "那",
            "个",
            "些",
            "之",
            "为",
            "以",
            "因",
            "所",
            "可以",
            "能够",
        }
        text = re.sub(r"[^\w\u4e00-\u9fff]", " ", text)
        tokens = []
        n = len(text)
        i = 0
        while i < n:
            if text[i] == " ":
                i += 1
                continue
            if i + 1 < n:
                two_char = text[i : i + 2]
                if two_char in stop_words:
                    i += 2
                    continue
            single_char = text[i]
            if single_char not in stop_words:
                tokens.append(single_char)
            i += 1
        return tokens

    def _load_or_build_vocab(self):
        row = self.cursor.execute(
            "SELECT COUNT(*) as cnt FROM newspaper_content_vocab"
        ).fetchone()
        if row["cnt"] > 0:
            rows = self.cursor.execute(
                "SELECT word, idx FROM newspaper_content_vocab ORDER BY idx"
            ).fetchall()
            self.vocab = {r["word"]: r["idx"] for r in rows}
        else:
            self._build_vocab()

    def _build_vocab(self):
        logger.info("Building content vocabulary...")
        rows = self.cursor.execute(
            "SELECT content FROM newspaper_articles WHERE content IS NOT NULL"
        ).fetchall()
        word_count: Dict[str, int] = {}
        for row in rows:
            tokens = self._tokenize(row["content"])
            for token in tokens:
                word_count[token] = word_count.get(token, 0) + 1
        sorted_words = sorted(word_count.items(), key=lambda x: -x[1])[:10000]
        self.vocab = {word: i for i, (word, _) in enumerate(sorted_words)}
        for word, idx in self.vocab.items():
            self.cursor.execute(
                "INSERT INTO newspaper_content_vocab (word, idx) VALUES (?, ?)",
                (word, idx),
            )
        self.conn.commit()
        logger.info(f"Content vocabulary built: {len(self.vocab)} words")

    def get_embedding(self, text: str) -> Optional[List[float]]:
        tokens = self._tokenize(text)
        dim = len(self.vocab)
        vector = [0.0] * dim
        token_count: Dict[str, int] = {}
        for token in tokens:
            token_count[token] = token_count.get(token, 0) + 1
        max_count = max(token_count.values()) if token_count else 1
        for token, count in token_count.items():
            if token in self.vocab:
                vector[self.vocab[token]] = count / max_count
        return vector

    def vectorize_all(self, force: bool = False):
        query = "SELECT id, content FROM newspaper_articles WHERE content IS NOT NULL"
        if not force:
            query += " AND id NOT IN (SELECT article_id FROM newspaper_content_vector)"
        query += " ORDER BY id"
        rows = self.cursor.execute(query).fetchall()
        logger.info(f"Content to vectorize: {len(rows)}")
        for i, row in enumerate(rows, 1):
            vector = self.get_embedding(row["content"])
            if vector:
                vector_blob = json.dumps(vector).encode("utf-8")
                self.cursor.execute(
                    """INSERT OR REPLACE INTO newspaper_content_vector
                    (article_id, content, vector, vector_dim, updated_at)
                    VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)""",
                    (row["id"], row["content"], vector_blob, len(vector)),
                )
                if i % 100 == 0:
                    self.conn.commit()
        self.conn.commit()
        logger.info(f"Content vectorization done! Processed {len(rows)} contents")

    def close(self):
        self.conn.close()


def run_all(
    newspapers: List[str] = None,
    year: int = 2026,
    start_month: int = 1,
    end_month: int = 4,
):
    if newspapers is None:
        newspapers = ["zsrb", "zswb"]

    for newspaper in newspapers:
        logger.info(f"=== Processing {newspaper.upper()} ===")
        scraper = NewspaperScraper(newspaper)
        scraper.scrape_year(year, start_month, end_month)

        vectorizer = TitleVectorizer(scraper.db_path)
        vectorizer.vectorize_all()
        vectorizer.close()

        content_vectorizer = ContentVectorizer(scraper.db_path)
        content_vectorizer.vectorize_all()
        content_vectorizer.close()

    logger.info("=== ALL DONE ===")


def revectorize_all_titles(newspapers: List[str] = None):
    if newspapers is None:
        newspapers = ["zsrb", "zswb"]

    for newspaper in newspapers:
        config = NewspaperScraper.NEWSPAPERS[newspaper]
        logger.info(f"=== Re-vectorizing titles for {config['name']} ===")
        vectorizer = TitleVectorizer(config["db_path"])
        vectorizer.vectorize_all(force=True)
        vectorizer.close()
    logger.info("=== Title re-vectorization complete ===")


def revectorize_all_content(newspapers: List[str] = None):
    if newspapers is None:
        newspapers = ["zsrb", "zswb"]

    for newspaper in newspapers:
        config = NewspaperScraper.NEWSPAPERS[newspaper]
        logger.info(f"=== Re-vectorizing content for {config['name']} ===")
        vectorizer = ContentVectorizer(config["db_path"])
        vectorizer.vectorize_all(force=True)
        vectorizer.close()
    logger.info("=== Content re-vectorization complete ===")


def revectorize_all(newspapers: List[str] = None):
    revectorize_all_titles(newspapers)
    revectorize_all_content(newspapers)
    logger.info("=== Complete re-vectorization done ===")


if __name__ == "__main__":
    import argparse
    from datetime import date

    today = date.today()
    default_year = today.year
    default_month = today.month
    default_day = today.day

    parser = argparse.ArgumentParser(description="Newspaper Scraper & Vectorizer")
    parser.add_argument(
        "--newspaper",
        "-n",
        choices=["zsrb", "zswb", "both"],
        default="both",
        help="Which newspaper to process (zsrb=日报, zswb=晚报, both=两个)",
    )
    parser.add_argument(
        "--year", "-y", type=int, default=default_year, help="Year to scrape"
    )
    parser.add_argument(
        "--start-month", "-s", type=int, default=default_month, help="Start month"
    )
    parser.add_argument(
        "--end-month", "-e", type=int, default=default_month, help="End month"
    )
    parser.add_argument(
        "--day", "-d", type=int, default=default_day, help="Day (for single day scrape)"
    )
    parser.add_argument(
        "--revectorize-title",
        action="store_true",
        help="Re-vectorize all titles (no scraping)",
    )
    parser.add_argument(
        "--revectorize-content",
        action="store_true",
        help="Re-vectorize all content (no scraping)",
    )
    parser.add_argument(
        "--revectorize-all",
        action="store_true",
        help="Re-vectorize titles AND content (no scraping)",
    )

    args = parser.parse_args()

    newspapers = None
    if args.newspaper == "zsrb":
        newspapers = ["zsrb"]
    elif args.newspaper == "zswb":
        newspapers = ["zswb"]
    else:
        newspapers = ["zsrb", "zswb"]

    if args.revectorize_title:
        revectorize_all_titles(newspapers)
    elif args.revectorize_content:
        revectorize_all_content(newspapers)
    elif args.revectorize_all:
        revectorize_all(newspapers)
    elif args.start_month == args.end_month and args.day:
        year, month, day = args.year, args.start_month, args.day
        for newspaper in newspapers:
            logger.info(
                f"=== Processing {newspaper.upper()} for {year}-{month:02d}-{day:02d} ==="
            )
            scraper = NewspaperScraper(newspaper)
            url_date = f"{year}-{month:02d}/{day:02d}"
            publication_date = f"{year}-{month:02d}-{day:02d}"
            scraper._create_database_tables()
            scraper._scrape_single_date(publication_date, url_date)
            vectorizer = TitleVectorizer(scraper.db_path)
            vectorizer.vectorize_all()
            vectorizer.close()

            content_vectorizer = ContentVectorizer(scraper.db_path)
            content_vectorizer.vectorize_all()
            content_vectorizer.close()
        logger.info("=== ALL DONE ===")
    else:
        run_all(newspapers, args.year, args.start_month, args.end_month)
