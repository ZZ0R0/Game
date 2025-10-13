use std::collections::VecDeque;
use std::time::{Duration, Instant};
use voxel_engine::generator_metrics::GeneratorStats;

/// Moniteur de performances en temps réel pour l'overlay
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    // Historique des mesures (dernière seconde)
    frame_times: VecDeque<f32>,
    generation_times: VecDeque<f32>,
    meshing_times: VecDeque<f32>,
    
    // Compteurs pour la dernière seconde
    chunks_generated_last_sec: VecDeque<(Instant, usize)>,
    chunks_meshed_last_sec: VecDeque<(Instant, usize)>,
    
    // Temps des opérations en cours
    current_generation_time: f32,
    current_meshing_time: f32,
    
    // Stats calculées (mises à jour chaque frame)
    pub avg_fps: f32,
    pub avg_frame_time_ms: f32,
    pub avg_generation_time_ms: f32,
    pub avg_meshing_time_ms: f32,
    pub chunks_per_sec_generated: f32,
    pub chunks_per_sec_meshed: f32,
    pub estimated_worker_idle_time_ms: f32,
    
    // Nouveaux calculs demandés
    pub total_generation_work_per_sec_ms: f32,
    pub total_meshing_work_per_sec_ms: f32,
    pub generation_work_per_worker_ms: f32,
    pub meshing_work_per_worker_ms: f32,
    
    // Compteurs totaux
    pub total_chunks_loaded: usize,
    pub total_chunks_rendered: usize,
    pub total_chunks_culled: usize,
    pub total_draw_calls: usize,
    pub jobs_pending: usize,
    
    last_update: Instant,
    worker_count: usize,
}

impl PerformanceMonitor {
    pub fn new(worker_count: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60),
            generation_times: VecDeque::with_capacity(100),
            meshing_times: VecDeque::with_capacity(100),
            chunks_generated_last_sec: VecDeque::new(),
            chunks_meshed_last_sec: VecDeque::new(),
            
            current_generation_time: 0.0,
            current_meshing_time: 0.0,
            
            avg_fps: 0.0,
            avg_frame_time_ms: 0.0,
            avg_generation_time_ms: 0.0,
            avg_meshing_time_ms: 0.0,
            chunks_per_sec_generated: 0.0,
            chunks_per_sec_meshed: 0.0,
            estimated_worker_idle_time_ms: 0.0,
            
            total_generation_work_per_sec_ms: 0.0,
            total_meshing_work_per_sec_ms: 0.0,
            generation_work_per_worker_ms: 0.0,
            meshing_work_per_worker_ms: 0.0,
            
            total_chunks_loaded: 0,
            total_chunks_rendered: 0,
            total_chunks_culled: 0,
            total_draw_calls: 0,
            jobs_pending: 0,
            
            last_update: Instant::now(),
            worker_count,
        }
    }
    
    /// Ajouter un temps de frame
    pub fn add_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push_back(frame_time_ms);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
    }
    
    /// Ajouter un temps de génération de chunk
    pub fn add_generation_time(&mut self, time_ms: f32, chunk_count: usize) {
        if time_ms > 0.0 {
            self.generation_times.push_back(time_ms);
            if self.generation_times.len() > 100 {
                self.generation_times.pop_front();
            }
        }
        
        if chunk_count > 0 {
            let now = Instant::now();
            self.chunks_generated_last_sec.push_back((now, chunk_count));
            self.current_generation_time = time_ms;
        }
    }
    
    /// Ajouter un temps de meshing de chunk
    pub fn add_meshing_time(&mut self, time_ms: f32, chunk_count: usize) {
        if time_ms > 0.0 {
            self.meshing_times.push_back(time_ms);
            if self.meshing_times.len() > 100 {
                self.meshing_times.pop_front();
            }
        }
        
        if chunk_count > 0 {
            let now = Instant::now();
            self.chunks_meshed_last_sec.push_back((now, chunk_count));
            self.current_meshing_time = time_ms;
        }
    }
    
    /// Mettre à jour les stats de rendu
    pub fn update_render_stats(&mut self, loaded: usize, rendered: usize, culled: usize, draw_calls: usize, pending: usize) {
        self.total_chunks_loaded = loaded;
        self.total_chunks_rendered = rendered;
        self.total_chunks_culled = culled;
        self.total_draw_calls = draw_calls;
        self.jobs_pending = pending;
    }
    
    /// Calculer toutes les statistiques moyennes
    pub fn update_stats(&mut self) {
        let now = Instant::now();
        let one_sec_ago = now - Duration::from_secs(1);
        
        // Nettoyer les anciens échantillons (> 1 seconde)
        self.chunks_generated_last_sec.retain(|(time, _)| *time > one_sec_ago);
        self.chunks_meshed_last_sec.retain(|(time, _)| *time > one_sec_ago);
        
        // FPS moyen
        if !self.frame_times.is_empty() {
            self.avg_frame_time_ms = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.avg_fps = 1000.0 / self.avg_frame_time_ms.max(0.1);
        }
        
        // Temps de génération moyen
        if !self.generation_times.is_empty() {
            self.avg_generation_time_ms = self.generation_times.iter().sum::<f32>() / self.generation_times.len() as f32;
        }
        
        // Temps de meshing moyen
        if !self.meshing_times.is_empty() {
            self.avg_meshing_time_ms = self.meshing_times.iter().sum::<f32>() / self.meshing_times.len() as f32;
        }
        
        // Chunks générés par seconde
        self.chunks_per_sec_generated = self.chunks_generated_last_sec
            .iter()
            .map(|(_, count)| *count)
            .sum::<usize>() as f32;
            
        // Chunks meshés par seconde
        self.chunks_per_sec_meshed = self.chunks_meshed_last_sec
            .iter()
            .map(|(_, count)| *count)
            .sum::<usize>() as f32;
        
        // NOUVEAUX CALCULS DEMANDÉS
        // Travail total par seconde pour chaque type d'opération
        self.total_generation_work_per_sec_ms = self.chunks_per_sec_generated * self.avg_generation_time_ms;
        self.total_meshing_work_per_sec_ms = self.chunks_per_sec_meshed * self.avg_meshing_time_ms;
        
        // Travail par worker (somme des temps divisée par nombre de workers)
        self.generation_work_per_worker_ms = self.total_generation_work_per_sec_ms / self.worker_count as f32;
        self.meshing_work_per_worker_ms = self.total_meshing_work_per_sec_ms / self.worker_count as f32;
        
        // CORRECTION: Estimation du temps d'idle des workers
        // Temps d'idle = temps disponible - temps utilisé, par worker
        let total_work_per_worker_ms = self.generation_work_per_worker_ms + self.meshing_work_per_worker_ms;
        let available_time_per_worker_ms = 1000.0; // 1 seconde = 1000ms
        self.estimated_worker_idle_time_ms = (available_time_per_worker_ms - total_work_per_worker_ms).max(0.0);
        
        self.last_update = now;
    }
    
    /// Générer le texte pour l'overlay
    pub fn get_overlay_text(&self) -> String {
        format!(
            "🔧 PERFORMANCE MONITOR\n\
            ═══════════════════════\n\
            📊 FPS: {:.1} ({:.1}ms/frame)\n\
            \n\
            🏗️  CHUNK PROCESSING:\n\
            • Generated: {:.1}/sec (avg: {:.1}ms)\n\
            • Meshed: {:.1}/sec (avg: {:.1}ms)\n\
            \n\
            ⚙️  WORKER LOAD:\n\
            • Gen work/worker: {:.1}ms/sec\n\
            • Mesh work/worker: {:.1}ms/sec\n\
            • Total work/worker: {:.1}ms/sec\n\
            • Worker idle: {:.1}ms/sec\n\
            \n\
            🎮 RENDERING:\n\
            • Loaded: {} chunks\n\
            • Rendered: {} ({} culled)\n\
            • Draw calls: {}\n\
            • Jobs pending: {}\n\
            \n\
            ⚡ WORKERS: {} threads",
            self.avg_fps,
            self.avg_frame_time_ms,
            self.chunks_per_sec_generated,
            self.avg_generation_time_ms,
            self.chunks_per_sec_meshed,
            self.avg_meshing_time_ms,
            self.generation_work_per_worker_ms,
            self.meshing_work_per_worker_ms,
            self.generation_work_per_worker_ms + self.meshing_work_per_worker_ms,
            self.estimated_worker_idle_time_ms,
            self.total_chunks_loaded,
            self.total_chunks_rendered,
            self.total_chunks_culled,
            self.total_draw_calls,
            self.jobs_pending,
            self.worker_count
        )
    }

    /// Générer le texte complet avec les détails du générateur
    pub fn get_overlay_text_with_generator(&self, gen_stats: &GeneratorStats) -> String {
        format!(
            "🔧 PERFORMANCE MONITOR\n\
            ═══════════════════════\n\
            📊 FPS: {:.1} ({:.1}ms/frame)\n\
            \n\
            🏗️  CHUNK PROCESSING:\n\
            • Generated: {:.1}/sec (avg: {:.1}ms)\n\
            • Meshed: {:.1}/sec (avg: {:.1}ms)\n\
            \n\
            ⚙️  WORKER LOAD:\n\
            • Gen work/worker: {:.1}ms/sec\n\
            • Mesh work/worker: {:.1}ms/sec\n\
            • Total work/worker: {:.1}ms/sec\n\
            • Worker idle: {:.1}ms/sec\n\
            \n\
            🔨 GENERATOR DETAIL:\n\
            • Underground ratio: {:.1}%\n\
            • Avg time per chunk: {:.3}ms\n\
            \n\
            ⏱️  Generator phases (avg):\n\
            • Underground check: {:.3}ms\n\
            • Underground fill: {:.3}ms\n\
            • Height calc: {:.3}ms\n\
            • Block placement: {:.3}ms\n\
            \n\
            💼 Generator work/sec:\n\
            • Underground check: {:.1}ms\n\
            • Underground fill: {:.1}ms\n\
            • Height calc: {:.1}ms\n\
            • Block placement: {:.1}ms\n\
            • Total: {:.1}ms\n\
            \n\
            👷 Generator work/worker:\n\
            • Underground check: {:.1}ms\n\
            • Underground fill: {:.1}ms\n\
            • Height calc: {:.1}ms\n\
            • Block placement: {:.1}ms\n\
            • Total: {:.1}ms\n\
            \n\
            🎮 RENDERING:\n\
            • Loaded: {} chunks\n\
            • Rendered: {} ({} culled)\n\
            • Draw calls: {}\n\
            • Jobs pending: {}\n\
            \n\
            ⚡ WORKERS: {} threads",
            // Main stats
            self.avg_fps,
            self.avg_frame_time_ms,
            self.chunks_per_sec_generated,
            self.avg_generation_time_ms,
            self.chunks_per_sec_meshed,
            self.avg_meshing_time_ms,
            self.generation_work_per_worker_ms,
            self.meshing_work_per_worker_ms,
            self.generation_work_per_worker_ms + self.meshing_work_per_worker_ms,
            self.estimated_worker_idle_time_ms,
            // Generator detail
            gen_stats.underground_ratio * 100.0,
            gen_stats.avg_total_ms,
            gen_stats.avg_underground_check_ms,
            gen_stats.avg_underground_fill_ms,
            gen_stats.avg_height_calc_ms,
            gen_stats.avg_block_placement_ms,
            gen_stats.total_underground_check_work_ms,
            gen_stats.total_underground_fill_work_ms,
            gen_stats.total_height_calc_work_ms,
            gen_stats.total_block_placement_work_ms,
            gen_stats.total_generation_work_ms,
            gen_stats.underground_check_work_per_worker_ms,
            gen_stats.underground_fill_work_per_worker_ms,
            gen_stats.height_calc_work_per_worker_ms,
            gen_stats.block_placement_work_per_worker_ms,
            gen_stats.total_work_per_worker_ms,
            // Rendering stats
            self.total_chunks_loaded,
            self.total_chunks_rendered,
            self.total_chunks_culled,
            self.total_draw_calls,
            self.jobs_pending,
            self.worker_count
        )
    }
}