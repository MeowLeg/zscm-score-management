-- 新闻
-- drop table article;
create table
    if not exists article (
        id integer primary key autoincrement,
        title text not null,
        tv_or_paper integer not null default 0, -- 0: 舟山新闻; 1: 舟山日报
        content text not null default '', -- new
        html_content text not null default '', -- new
        ref_id integer not null default 0, -- new
        -- 发布日期
        publish_year integer not null,
        publish_month integer not null,
        publish_day integer not null,
        -- 电视
        tv_url text default '',
        duration integer not null default 0, -- new
        -- 报纸
        page_meta_id integer not null default 0,
        page_name text not null default '',
        character_count integer not null default 0, -- new
        -- 分数
        score_basic integer not null default 0,
        score_action integer not null default 0,
        is_collaboration integer not null default 0, -- 0: 非通讯员稿件; 1: 通讯员稿件
        state integer not null default 1
    );

create table if not exists program (
    id integer primary key autoincrement,
    name text notn null default '',
    media_type integer not null default 0, -- 0: 电视; 1: 报纸
    site_id integer not null default 0, -- 0: 舟山新闻; 1: 舟山日报
    parent_id integer, -- 根节点
    code text not null default '',
    state integer not null default 1
);
-- insert into program (name, media_type, site_id, code)
-- values
--     ('舟山新闻', 0, 0, '590f6b165afa45f1bd8fc1ed31756fb7'),
--     ('舟山日报', 1, 1, '');

-- insert into article (title, tv_or_paper, publish_year, publish_month, publish_day, tv_url, page_meta_id, page_name, score_basic, score_action, state)
-- values
-- -- news
-- ('【闪亮“十四五”·我的这五年】朱雨雷：见证舟山船舶产业“智” 变', 0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 450, 100, 1),
-- ('我市各地学习贯彻党的二十届四中全会精神',                  0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('普陀山国际机场启用综合服务中心 为入境旅客提供“一站式” 服务', 0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('监管部门“亮码入企” 执法全程留痕可溯',                    0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('嵊泗海关优化服务 助贻贝“游”向世界餐桌',                  0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('2025年中国飞镖公开赛（浙江舟山站）开赛',                 0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('全市职工定向徒步活动举行',                             0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('【我的乡创新天地】李洁：以咖啡香为引 打开创意新空间',      0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('国家地理标志赋能 定海皋泄香柚销售旺',                    0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('11月1日起国产小客车新车上牌全程数字化办理',              0, 2025, 12, 1, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('何中伟在新城调研甬东勾山和小干岛开发建设情况时强调 认真贯彻落实党的二十届四中全会精神 高标准建设现代化海上花园城市', 0, 2025, 12, 2, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('我市各部门各单位学习贯彻党的二十届四中全会精神',              0, 2025, 12, 2, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('2025年中国·舟山群岛铁人三项精英赛鸣笛开赛 近千名选手逐浪比拼', 0, 2025, 12, 2, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- ('【新闻链接】铁人三项精英赛：一场赛事串起一个消费链条',         0, 2025, 12, 2, 'https://mc-public-kh.wifizs.cn/16290457183556_hd.mp4?width=1280&height=720&isVertical=0&fsize=50990685&duration=196.0', 0, '', 300, 100, 1),
-- -- daily
-- ('舟山文旅项目提前超额完成年度投资目标',                       1, 2025, 12, 1, '', 1, '舟山日报', 300, 100, 1),
-- ('“舶赛通”平台助力航运企业数字化跃迁',                        1, 2025, 12, 1, '', 1, '舟山日报', 300, 100, 1),
-- ('以实干实绩实效高水平建设现代海洋城市',                       1, 2025, 12, 1, '', 1, '舟山日报', 300, 100, 1),
-- ('舟山企业将参与采购展示洽谈等环节',                          1, 2025, 12, 1, '', 1, '舟山日报', 300, 100, 1),
-- ('认真贯彻落实党的二十届四中全会精神高标准建设现代化海上花园城市', 1, 2025, 12, 2, '', 2, '舟山日报', 300, 100, 1),
-- ('普陀山国际机场国际服务中心昨投运',                          1, 2025, 12, 2, '', 2, '舟山日报', 300, 100, 1),
-- ('我市实施养老护理员补助新政',                               1, 2025, 12, 2, '', 2, '舟山日报', 300, 100, 1),
-- ('聚力实施“985”行动 加快建设现代海洋城市',                    1, 2025, 12, 2, '', 2, '舟山日报', 300, 100, 1),
-- ('改革开放　动能澎湃',                                      1, 2025, 12, 2, '', 2, '舟山日报', 300, 100, 1);
-- zsnews_article.score_basic + zsnews_article.score_action = sum(zsnews_article_reporter_score.score)
create table
    if not exists article_reporter_score (
        id integer primary key autoincrement,
        article_id integer not null,
        reporter_id integer not null,
        reporter_category_id integer not null default 3, -- 可动态修改记者属性
        score integer not null default 0
    );

-- drop table reporter;
create table
    if not exists reporter (
        id integer primary key autoincrement,
        name text not null,
        phone text not null default '',
        ref_code text unque,
        reporter_category_id integer not null default 3, -- 本职
        department text not null default '',
        state integer not null default 1
    );

-- -- 通讯员 该表是否需要存在，还需要观察
-- create table
--     if not exists collaborator (
--         id integer primary key autoincrement,
--         name text not null,
--         phone text not null default '',
--         ref_code text unque,
--         department text not null default '',
--         state integer not null default 1
--     );

create table
    if not exists reporter_category (
        id integer primary key autoincrement,
        name text not null
    );

create table
    if not exists event_score (
        id integer primary key autoincrement,
        reporter_id integer not null,
        content text not null,
        score integer not null default 0,
        score_from text not null default '',
        publish_year integer not null,
        publish_month integer not null,
        -- publish_day integer not null,
        state integer not null default 1 -- 仅用于判断是否需要物理删除
    );

create table
    if not exists cooperation_score (
        id integer primary key autoincrement,
        reporter_id integer not null,
        content text not null,
        score integer not null default 0,
        score_from text not null default '',
        publish_year integer not null,
        publish_month integer not null,
        -- publish_day integer not null,
        state integer not null default 1
    );

create table
    if not exists monthly_sub_score (
        id integer primary key autoincrement,
        reporter_id integer not null,
        score integer not null default 0,
        reason text not null default '',
        publish_year integer not null,
        publish_month integer not null,
        state integer not null default 1,
        -- publish_day integer not null,
        unique (reporter_id, publish_year, publish_month)
    );

create table
    if not exists monthly_add_score (
        id integer primary key autoincrement,
        reporter_id integer not null,
        score integer not null default 0,
        reason text not null default '',
        publish_year integer not null,
        publish_month integer not null,
        state integer not null default 1,
        -- publish_day integer not null,
        unique (reporter_id, publish_year, publish_month)
    );

-- 舟山日报
create table
    if not exists page_meta (
        id integer primary key autoincrement,
        paper_name text not null, -- 报纸名称
        page_no text not null, -- 版面号
        page_web_no text not null default '' -- 版面网络号
    );

-- admin
create table
    if not exists admin (
        id integer primary key autoincrement,
        name text not null,
        password text not null
    );

create table 
    if not exists admin_sites (
        id integer primary key autoincrement,
        admin_id integer not null default 0,
        site_id integer not null default 0 -- 0: 舟山新闻; 1: 舟山日报
    );

insert into admin_sites (admin_id, site_id) values (4, 0);



-- insert into
--     admin (name, password)
-- values
--     ('jh', '5f8d5c1a8b3e9c7d6f5e4d3c2b1a0f9e');

-- insert into
--     event_score (
--         reporter_id,
--         content,
--         score,
--         score_from,
--         publish_year,
--         publish_month,
--         publish_day
--     )
-- values
--     (1, 'XX杀人事件', 300, '记者', 2025, 12, 1),
--     (2, 'XX杀人事件', 300, '记者', 2025, 12, 1),
--     (3, 'XX杀人事件', 300, '记者', 2025, 12, 1),
--     (4, 'XX杀人事件', 300, '记者', 2025, 12, 1),
--     (5, 'XX杀人事件', 300, '记者', 2025, 12, 2),
--     (6, 'XX杀人事件', 300, '记者', 2025, 12, 2),
--     (7, 'XX杀人事件', 300, '记者', 2025, 12, 2);
-- insert into page_meta (paper_name, page_no) values
--     ('舟山日报', '日报1版'),
--     ('舟山日报', '日报2版'),
--     ('舟山日报', '日报3版'),
--     ('舟山日报', '日报4版'),
--     ('舟山日报', '日报5版'),
--     ('舟山日报', '日报6版'),
--     ('舟山日报', '日报7版'),
--     ('舟山日报', '日报8版');
-- data
-- insert into
--     reporter_category (name)
-- values
--     ('主任'),
--     ('副主任'),
--     ('文字记者'),
--     ('摄像记者'),
--     ('摄影记者'),
--     ('主持人'),
--     ('通讯员');
-- insert into
--     reporter (name, phone, reporter_category_id)
-- values
--     -- 采访部
--     ('郑颖', '13515805716', 1),
--     ('刘浩', '13957212033', 2),
--     ('应科', '13957220330', 2),
--     ('王晓东', '13705808058', 3),
--     ('姚凯乐', '13675801610', 3),
--     ('王涵真', '18758325356', 3),
--     ('方琦', '13675808986', 3),
--     ('何斌', '13655802060', 3),
--     ('吴娜娜', '18768054695', 3),
--     ('丛琳', '18768053875', 3),
--     ('林家豪', '15858067867', 3),
--     ('陈逸麟', '15268002714', 3),
--     ('沈磊', '13906802122', 3),
--     ('张磊', '13758015307', 3),
--     ('戎浩', '18768075350', 3),
--     ('宓妤霖', '18857074410', 3),
--     ('孙璐艳', '13656828839', 3),
--     ('刘凯', '15068004101', 3),
--     ('刘晓梦', '18358077610', 3),
--     ('邢楠', '19518062106', 3),
--     ('顾嘉昊', '13957215362', 3),
--     ('范家乐', '18768057594', 3),
--     ('王跃蒙', '15705805288', 3),
--     ('张曦', '13868212819', 3),
--     ('林增光', '13454080803', 3),
--     ('董武', '13616805794', 3),
--     ('陆炳', '13506805426', 3),
--     ('方圆', '13735005734', 3),
--     ('陈雨露', '18158021510', 3),
--     ('颜榕', '13705807071', 3),
--     ('金蒙兰', '17816548879', 3),
--     ('陶思源', '15905801002', 3),
--     ('董凌浩', '18358097270', 3),
--     ('陈妤', '13567678103', 3),
--     ('贺贤挺', '13905803220', 3),
--     -- 时政要闻
--     ('徐祝君', '13567678029', 1),
--     ('方智斌', '13758027657', 2),
--     ('黄燕玲', '18205806204', 3),
--     ('虞仁珂', '13587049946', 3),
--     ('陈颖丹', '13666597116', 3),
--     ('张莉莉', '18768070934', 3),
--     ('葛高蓉', '17888835377', 3),
--     -- 经济专题部
--     ('曹玲', '13515803971', 1),
--     ('石艳虹', '15205803652', 2),
--     ('邱波彤', '13567678038', 3),
--     ('何菁', '13857209872', 3),
--     ('翁雨薇', '13587086927', 3),
--     ('王倩倩', '13857221205', 3),
--     -- 民生专题部
--     ('金春玲', '13957221970', 3),
--     ('汪超群', '15205804836', 3),
--     ('陈斌娜', '13867208818', 3),
--     ('朱丽媛', '13646808070', 3),
--     ('岑瑜', '13957215000', 3);
