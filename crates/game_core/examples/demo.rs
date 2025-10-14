// Example demonstrating the basic functionality of game_core
use game_core::{factory, world::GameWorld, objects::Position};

fn main() {
    println!("=== Space Engineers Clone - Core Demo ===\n");

    // Create a new game world
    let mut world = GameWorld::new();
    
    // Create some players and ships
    println!("1. Creating players and ships...");
    let spawn_pos = Position::new(0.0, 0.0, 0.0);
    let (player1_id, ship1_id) = world.spawn_new_player("Alice", spawn_pos);
    
    let spawn_pos2 = Position::new(100.0, 0.0, 100.0);
    let (player2_id, ship2_id) = world.spawn_new_player("Bob", spawn_pos2);
    
    println!("   - Alice spawned with ID {} and ship ID {}", player1_id, ship1_id);
    println!("   - Bob spawned with ID {} and ship ID {}", player2_id, ship2_id);

    // Show ship details
    println!("\n2. Ship Details:");
    if let Some(ship) = world.ships.get(&ship1_id) {
        println!("   - Alice's ship '{}' has {} blocks, mass: {:.1} kg", 
                ship.grid.name, ship.large_blocks.len(), ship.total_mass());
        let pos = ship.get_position();
        println!("   - Position: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
    }

    // Show player details
    println!("\n3. Player Details:");
    if let Some(player) = world.players.get(&player1_id) {
        println!("   - {} - Health: {:.1}%, Oxygen: {:.1}%, Energy: {:.1}%", 
                player.name, player.health, player.oxygen, player.energy);
        let pos = player.get_position();
        println!("   - Position: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
    }

    // Simulate some time passing
    println!("\n4. Simulating 30 seconds of gameplay...");
    world.update(30.0);

    // Check player status after simulation
    if let Some(player) = world.players.get(&player1_id) {
        println!("   - Alice after 30s - Health: {:.1}%, Oxygen: {:.1}%, Energy: {:.1}%", 
                player.health, player.oxygen, player.energy);
        
        if player.needs_oxygen() {
            println!("   - ⚠️  Alice needs oxygen!");
        }
        if player.needs_energy() {
            println!("   - ⚠️  Alice needs energy!");
        }
    }

    // Test ship modification
    println!("\n5. Testing ship modifications...");
    if let Some(ship) = world.ships.get_mut(&ship1_id) {
        let new_block = game_core::objects::LargeBlock::default_armor(ship.next_block_id());
        ship.add_block(new_block);
        println!("   - Added armor block to Alice's ship");
        println!("   - Ship now has {} blocks, mass: {:.1} kg", 
                ship.large_blocks.len(), ship.total_mass());

        // Test damage
        let damaged = ship.damage_block(0, 50.0);
        println!("   - Applied 50 damage to block 0, destroyed: {}", damaged);
    }

    println!("\n=== Demo completed successfully! ===");
}