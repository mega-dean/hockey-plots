CREATE TABLE divisions (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE last_period_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE scores (
    id INTEGER PRIMARY KEY,
    home INTEGER NOT NULL,
    away INTEGER NOT NULL,
    last_period_type_id INTEGER NOT NULL,
    FOREIGN KEY (last_period_type_id) REFERENCES last_period_types (id)
);

CREATE TABLE teams (
    id INTEGER PRIMARY KEY,
    api_id INTEGER NOT NULL,
    abbrev TEXT NOT NULL,
    division_id INTEGER NOT NULL,
    r INTEGER NOT NULL,
    g INTEGER NOT NULL,
    b INTEGER NOT NULL,
    FOREIGN KEY (division_id) REFERENCES divisions (id)
);

CREATE TABLE games (
    id INTEGER PRIMARY KEY,
    api_id INTEGER NOT NULL UNIQUE,
    home_team_id INTEGER NOT NULL,
    away_team_id INTEGER NOT NULL,
    score_id INTEGER,
    FOREIGN KEY (home_team_id) REFERENCES teams (id),
    FOREIGN KEY (away_team_id) REFERENCES teams (id),
    FOREIGN KEY (score_id) REFERENCES scores (id)
);

INSERT INTO divisions (name) VALUES ('Metropolitan'), ('Atlantic'), ('Central'), ('Pacific');
INSERT INTO last_period_types (name) VALUES ('Regulation'), ('Overtime'), ('Shootout');

INSERT INTO teams (api_id, abbrev, division_id, r, g, b) VALUES
    (24, 'ANA', (SELECT id FROM divisions WHERE name = 'Pacific'), 252, 76, 2),
    (20, 'CGY', (SELECT id FROM divisions WHERE name = 'Pacific'), 210, 0, 28),
    (22, 'EDM', (SELECT id FROM divisions WHERE name = 'Pacific'), 4, 30, 66),
    (26, 'LAK', (SELECT id FROM divisions WHERE name = 'Pacific'), 162, 170, 173),
    (55, 'SEA', (SELECT id FROM divisions WHERE name = 'Pacific'), 104, 162, 185),
    (28, 'SJS', (SELECT id FROM divisions WHERE name = 'Pacific'), 0, 109, 117),
    (23, 'VAN', (SELECT id FROM divisions WHERE name = 'Pacific'), 0, 32, 91),
    (54, 'VGK', (SELECT id FROM divisions WHERE name = 'Pacific'), 185, 151, 91),
    (53, 'ARI', (SELECT id FROM divisions WHERE name = 'Central'), 140, 38, 51),
    (16, 'CHI', (SELECT id FROM divisions WHERE name = 'Central'), 207, 10, 44),
    (21, 'COL', (SELECT id FROM divisions WHERE name = 'Central'), 111, 38, 61),
    (25, 'DAL', (SELECT id FROM divisions WHERE name = 'Central'), 0, 104, 71),
    (30, 'MIN', (SELECT id FROM divisions WHERE name = 'Central'), 2, 73, 48),
    (18, 'NSH', (SELECT id FROM divisions WHERE name = 'Central'), 255, 184, 28),
    (19, 'STL', (SELECT id FROM divisions WHERE name = 'Central'), 0, 47, 135),
    (52, 'WPG', (SELECT id FROM divisions WHERE name = 'Central'), 4, 30, 66),
    (6, 'BOS', (SELECT id FROM divisions WHERE name = 'Atlantic'), 252, 181, 20),
    (7, 'BUF', (SELECT id FROM divisions WHERE name = 'Atlantic'), 0, 48, 135),
    (17, 'DET', (SELECT id FROM divisions WHERE name = 'Atlantic'), 206, 17, 38),
    (13, 'FLA', (SELECT id FROM divisions WHERE name = 'Atlantic'), 185, 151, 91),
    (8, 'MTL', (SELECT id FROM divisions WHERE name = 'Atlantic'), 175, 30, 45),
    (9, 'OTT', (SELECT id FROM divisions WHERE name = 'Atlantic'), 183, 146, 87),
    (14, 'TBL', (SELECT id FROM divisions WHERE name = 'Atlantic'), 0, 40, 104),
    (10, 'TOR', (SELECT id FROM divisions WHERE name = 'Atlantic'), 0, 32, 91),
    (12, 'CAR', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 206, 17, 38),
    (29, 'CBJ', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 0, 38, 84),
    (1, 'NJD', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 206, 17, 38),
    (2, 'NYI', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 0, 83, 155),
    (3, 'NYR', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 0, 56, 168),
    (4, 'PHI', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 247, 73, 2),
    (5, 'PIT', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 252, 181, 20),
    (15, 'WSH', (SELECT id FROM divisions WHERE name = 'Metropolitan'), 200, 16, 46);
