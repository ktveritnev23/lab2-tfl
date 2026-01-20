use crate::automata::DFA;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, Write};

// Структура для хранения результатов
pub struct DistinguishingTable {
    pub table: HashMap<(usize, usize), String>, // Различающие строки для пар состояний
    pub equivalent_pairs: HashSet<(usize, usize)>, // Пары эквивалентных состояний
    pub is_minimal: bool, // Является ли ДКА минимальным
}

impl DistinguishingTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            equivalent_pairs: HashSet::new(),
            is_minimal: true,
        }
    }
    
    // Получить различающую строку для пары состояний
    pub fn get_distinguishing_string(&self, p: usize, q: usize) -> Option<&String> {
        let key = if p < q { (p, q) } else { (q, p) };
        self.table.get(&key)
    }
    
    // Проверить, эквивалентны ли состояния
    pub fn are_equivalent(&self, p: usize, q: usize) -> bool {
        let key = if p < q { (p, q) } else { (q, p) };
        !self.table.contains_key(&key) && p != q
    }
    
    // Вывести таблицу в консоль
    pub fn print_table(&self, states: &[usize]) {
        println!("\nТаблица различающих строк:");
        print!("     ");
        for &state in states {
            print!("{:4} ", state);
        }
        println!();
        
        for &row_state in states {
            print!("{:4} ", row_state);
            for &col_state in states {
                if row_state == col_state {
                    print!("  -  ");
                } else if self.are_equivalent(row_state, col_state) {
                    print!(" Экв ");
                } else if let Some(s) = self.get_distinguishing_string(row_state, col_state) {
                    if s.is_empty() {
                        print!("  ε  ");
                    } else {
                        print!("{:^5}", s);
                    }
                } else {
                    print!("  ?  ");
                }
            }
            println!();
        }
    }
}

// Основная функция построения таблицы различающих строк
pub fn build_distinguishing_table(dfa: &DFA) -> DistinguishingTable {
    let mut result = DistinguishingTable::new();
    
    // 1. Построение обратного отображения переходов
    // rev_map[destination][symbol] -> HashSet<source_states>
    let mut rev_map: HashMap<usize, HashMap<char, HashSet<usize>>> = HashMap::new();
    
    for (&(src, sym), &dst) in &dfa.transition {
        rev_map
            .entry(dst)
            .or_default()
            .entry(sym)
            .or_default()
            .insert(src);
    }
    
    // 2. Инициализация структур данных
    let states = &dfa.states;
    let final_states: HashSet<usize> = dfa.final_states.iter().cloned().collect();
    let alphabet = &dfa.alphabet;
    
    let mut dist_map: HashMap<(usize, usize), String> = HashMap::new();
    let mut queue: VecDeque<(usize, usize, String)> = VecDeque::new();
    
    // 3. Инициализация тривиально различимых пар
    for i in 0..states.len() {
        for j in i + 1..states.len() {
            let p = states[i];
            let q = states[j];
            
            let p_final = final_states.contains(&p);
            let q_final = final_states.contains(&q);
            
            // Если одно состояние финальное, а другое - нет
            if p_final != q_final {
                let key = (p, q);
                dist_map.insert(key, String::new()); // ε-строка
                queue.push_back((p, q, String::new()));
                result.table.insert(key, String::new());
            }
        }
    }
    
    // 4. Обратное распространение (BFS)
    while let Some((p, q, w)) = queue.pop_front() {
        for &sym in alphabet {
            // Получаем предшественников p и q по символу sym
            let preds_p = rev_map
                .get(&p)
                .and_then(|m| m.get(&sym))
                .map(|s| s.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_else(Vec::new);
            
            let preds_q = rev_map
                .get(&q)
                .and_then(|m| m.get(&sym))
                .map(|s| s.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_else(Vec::new);
            
            // Обрабатываем все пары предшественников
            for &p_prev in &preds_p {
                for &q_prev in &preds_q {
                    if p_prev == q_prev {
                        continue;
                    }
                    
                    // Упорядочиваем пару (меньший индекс первым)
                    let (u, v) = if p_prev < q_prev {
                        (p_prev, q_prev)
                    } else {
                        (q_prev, p_prev)
                    };
                    
                    // Если для этой пары еще не найдена различающая строка
                    if !dist_map.contains_key(&(u, v)) {
                        let new_w = format!("{}{}", sym, w);
                        dist_map.insert((u, v), new_w.clone());
                        queue.push_back((u, v, new_w.clone()));
                        result.table.insert((u, v), new_w);
                    }
                }
            }
        }
    }
    
    // 5. Определение эквивалентных пар и минимальности
    for i in 0..states.len() {
        for j in i + 1..states.len() {
            let p = states[i];
            let q = states[j];
            
            if !dist_map.contains_key(&(p, q)) {
                result.equivalent_pairs.insert((p, q));
                result.is_minimal = false;
            }
        }
    }
    
    result
}

// Функция для экспорта таблицы в файл (CSV формат)
pub fn export_table_to_csv(table: &DistinguishingTable, states: &[usize], filename: &str) -> io::Result<()> {
    let mut file = std::fs::File::create(filename)?;
    
    // Заголовок
    write!(file, "Состояние")?;
    for &state in states {
        write!(file, ",{}", state)?;
    }
    writeln!(file)?;
    
    // Данные
    for &row_state in states {
        write!(file, "{}", row_state)?;
        for &col_state in states {
            if row_state == col_state {
                write!(file, ",-")?;
            } else if table.are_equivalent(row_state, col_state) {
                write!(file, ",Экв")?;
            } else if let Some(s) = table.get_distinguishing_string(row_state, col_state) {
                if s.is_empty() {
                    write!(file, ",ε")?;
                } else {
                    write!(file, ",{}", s)?;
                }
            } else {
                write!(file, ",?")?;
            }
        }
        writeln!(file)?;
    }
    
    println!("Таблица сохранена в файл: {}", filename);
    Ok(())
}

// Добавим вспомогательный метод в impl DFA для проверки строки из конкретного состояния
impl DFA {
    pub fn accepts_from_state(&self, start_state: usize, input: &str) -> bool {
        let mut current_state = start_state;
        
        for ch in input.chars() {
            if !self.alphabet.contains(&ch) {
                return false;
            }
            
            if let Some(&next_state) = self.transition.get(&(current_state, ch)) {
                current_state = next_state;
            } else {
                return false;
            }
        }
        
        self.final_states.contains(&current_state)
    }
}