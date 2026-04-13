# encoding=utf-8
"""
相似标题搜索脚本
- 使用余弦相似度计算向量相似度
- 返回相似标题列表
"""

import sqlite3
import json
import re
import sys
from pathlib import Path
from typing import List, Dict, Optional, Tuple


class SimilarTitleSearcher:
    def __init__(self, db_path: str = "./zscm.db"):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self.cursor = self.conn.cursor()
        self.vocab: Dict[str, int] = {}
        self._load_vocab()

    def _load_vocab(self):
        """加载词汇表"""
        rows = self.cursor.execute("SELECT word, idx FROM vector_vocab ORDER BY idx").fetchall()
        self.vocab = {r["word"]: r["idx"] for r in rows}

    def _tokenize(self, text: str) -> List[str]:
        """简单的中文分词（按字符和常用词分割）"""
        text = re.sub(r"[^\w\u4e00-\u9fff]", " ", text)
        tokens = []
        n = len(text)
        i = 0
        while i < n:
            if text[i] == " ":
                i += 1
                continue
            if i + 1 < n and text[i:i+2] in ["的", "是", "在", "有", "和", "与", "及", "或", "但", "而", "也", "都", "就", "被", "把", "让", "给", "到", "从", "向", "对", "于", "等", "这", "那", "个", "些", "之", "为", "以", "因", "所", "可以", "能够", "已经", "正在", "将要", "不会", "没有", "不是", "就是", "还是", "或者", "并且", "但是", "然而", "因此", "所以", "因为", "由于", "虽然", "尽管", "即使", "就算", "只要", "只有", "除非", "无论", "不管", "凡是", "所有", "全部", "一切", "任何", "每个", "各个", "各种", "各类", "各项", "各种", "各位", "各位", "许多", "很多", "不少", "大量", "许多", "众多", "诸多", "若干", "一些", "一点", "少许", "稍微", "比较", "相当", "非常", "极其", "特别", "尤其", "更加", "越发", "最", "更", "较", "比", "像", "似", "如", "同", "跟", "和", "与", "及", "以及", "暨", "及于", "至于", "关于", "对于", "针对", "按照", "根据", "依据", "凭借", "通过", "经过", "经由", "由于", "因为", "鉴于", "出于", "由于", "多亏", "幸亏", "幸好", "好在", "可惜", "遗憾", "不幸", "幸运", "可喜", "可贺", "可悲", "可叹", "可怜", "可恨", "可恶", "可气", "可笑", "可悲", "可叹", "可怜", "可恨", "可恶", "可气", "可笑", "可喜", "可贺"]:
                tokens.append(text[i:i+2])
                i += 2
            else:
                tokens.append(text[i])
                i += 1
        return tokens

    def get_embedding(self, text: str) -> Optional[List[float]]:
        """获取文本向量（基于词汇表的 one-hot + TF 向量）"""
        tokens = self._tokenize(text)
        dim = len(self.vocab)
        vector = [0.0] * dim
        
        token_count: Dict[str, int] = {}
        for token in tokens:
            token_count[token] = token_count.get(token, 0) + 1
        
        max_count = max(token_count.values()) if token_count else 1
        
        for token, count in token_count.items():
            if token in self.vocab:
                idx = self.vocab[token]
                vector[idx] = count / max_count
        
        return vector

    def cosine_similarity(self, vec1: List[float], vec2: List[float]) -> float:
        """计算余弦相似度"""
        if len(vec1) != len(vec2):
            return 0.0
        
        dot_product = sum(a * b for a, b in zip(vec1, vec2))
        norm1 = sum(a * a for a in vec1) ** 0.5
        norm2 = sum(b * b for b in vec2) ** 0.5
        
        if norm1 == 0 or norm2 == 0:
            return 0.0
        
        return dot_product / (norm1 * norm2)

    def search(self, title: str, threshold: float = 0.5, limit: int = 10) -> List[Dict]:
        """搜索相似标题"""
        query_vector = self.get_embedding(title)
        if not query_vector:
            return []
        
        rows = self.cursor.execute("""
            SELECT 
                tv.id, tv.article_id, tv.title, tv.vector,
                a.publish_year, a.publish_month, a.publish_day,
                a.page_name, a.page_meta_id
            FROM title_vector tv
            JOIN article a ON tv.article_id = a.id
        """).fetchall()
        
        results = []
        for row in rows:
            vector_data = json.loads(row["vector"])
            similarity = self.cosine_similarity(query_vector, vector_data)
            
            if similarity >= threshold:
                results.append({
                    "id": row["id"],
                    "article_id": row["article_id"],
                    "title": row["title"],
                    "similarity": similarity,
                    "publish_year": row["publish_year"],
                    "publish_month": row["publish_month"],
                    "publish_day": row["publish_day"],
                    "page_name": row["page_name"],
                    "page_meta_id": row["page_meta_id"],
                })
        
        results.sort(key=lambda x: -x["similarity"])
        return results[:limit]

    def close(self):
        """关闭数据库连接"""
        self.conn.close()


def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Missing title parameter"}))
        return
    
    title = sys.argv[1]
    threshold = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    
    db_path = Path(__file__).resolve().parent.parent / "zscm.db"
    searcher = SimilarTitleSearcher(str(db_path))
    
    try:
        results = searcher.search(title, threshold, limit)
        print(json.dumps(results, ensure_ascii=False))
    finally:
        searcher.close()


if __name__ == "__main__":
    main()
