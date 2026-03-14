# encoding=utf-8

import sqlite3
import xlrd


class MergeReporter:
    def __init__(self, db_file, xls_file):
        self.conn = sqlite3.connect(db_file)
        self.cursor = self.conn.cursor()
        self.sheet = xlrd.open_workbook(xls_file).sheet_by_index(0)

    def job(self):
        for row in range(1, self.sheet.nrows):
            row_data = self.sheet.row_values(row)
            r = self.cursor.execute(
                "SELECT id FROM reporter WHERE name = ?",
                (row_data[2],)
            ).fetchone()
            if r is None:
                self.cursor.execute(
                    """
                    INSERT INTO reporter(
                        name, phone, ref_code, reporter_category_id, department
                    ) VALUES (?, ?, ?, ?, ?)
                    """,
                    (row_data[2], row_data[3], "", 3, row_data[4])
                )
        self.conn.commit()


if __name__ == "__main__":
    db_file = "../zscm.db"
    m = MergeReporter(db_file, "reporter.xls")
    m.job()