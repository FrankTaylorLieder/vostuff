use crate::auth::PasswordHasher;
use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct SampleDataLoader<'a> {
    pool: &'a PgPool,
}

pub struct SampleDataResult {
    pub coke_org_id: Uuid,
    pub pepsi_org_id: Uuid,
    pub bob_user_id: Uuid,
    pub alice_user_id: Uuid,
}

impl<'a> SampleDataLoader<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn load_sample_data(&self) -> Result<SampleDataResult> {
        println!("Loading sample data...");

        // Create organizations
        let coke_id = self.create_org("Coke", "Coca-Cola Organization").await?;
        let pepsi_id = self.create_org("Pepsi", "PepsiCo Organization").await?;

        // Create users with passwords
        // Bob is a regular user, Alice is an admin
        let bob_id = self
            .create_user("Bob", "bob@coke.com", Some("secret123"))
            .await?;
        let alice_id = self
            .create_user("Alice", "alice@pepsi.com", Some("secret123"))
            .await?;

        // Associate users with orgs and assign roles
        self.add_user_to_org(bob_id, coke_id, vec!["USER".to_string()])
            .await?;
        self.add_user_to_org(
            alice_id,
            pepsi_id,
            vec!["USER".to_string(), "ADMIN".to_string()],
        )
        .await?;

        // Create locations for each org
        let coke_locations = self.create_locations(coke_id).await?;
        let pepsi_locations = self.create_locations(pepsi_id).await?;

        // Create collections for each org
        let coke_collections = self.create_collections(coke_id).await?;
        let pepsi_collections = self.create_collections(pepsi_id).await?;

        // Create tags for each org
        self.create_tags(coke_id).await?;
        self.create_tags(pepsi_id).await?;

        // Create items for Coke
        println!("Creating items for Coke organization...");
        self.create_items_for_org(coke_id, &coke_locations, &coke_collections)
            .await?;

        // Create items for Pepsi
        println!("Creating items for Pepsi organization...");
        self.create_items_for_org(pepsi_id, &pepsi_locations, &pepsi_collections)
            .await?;

        println!("✓ Sample data loaded successfully!");

        Ok(SampleDataResult {
            coke_org_id: coke_id,
            pepsi_org_id: pepsi_id,
            bob_user_id: bob_id,
            alice_user_id: alice_id,
        })
    }

    async fn create_org(&self, name: &str, description: &str) -> Result<Uuid> {
        let rec = sqlx::query!(
            "INSERT INTO organizations (name, description) VALUES ($1, $2) RETURNING id",
            name,
            description
        )
        .fetch_one(self.pool)
        .await?;
        println!("  ✓ Created organization: {}", name);
        Ok(rec.id)
    }

    async fn create_user(
        &self,
        name: &str,
        identity: &str,
        password: Option<&str>,
    ) -> Result<Uuid> {
        let password_hash = match password {
            Some(pwd) => Some(PasswordHasher::hash_password(pwd)?),
            None => None,
        };

        let row = sqlx::query(
            "INSERT INTO users (name, identity, password_hash) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(name)
        .bind(identity)
        .bind(password_hash)
        .fetch_one(self.pool)
        .await?;

        let id: Uuid = row.get("id");
        println!("  ✓ Created user: {} ({})", name, identity);
        Ok(id)
    }

    async fn add_user_to_org(&self, user_id: Uuid, org_id: Uuid, roles: Vec<String>) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_organizations (user_id, organization_id, roles) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(org_id)
        .bind(&roles)
        .execute(self.pool)
        .await?;
        println!("  ✓ Added user to organization with roles: {:?}", roles);
        Ok(())
    }

    async fn create_locations(&self, org_id: Uuid) -> Result<Vec<Uuid>> {
        let locations = vec![
            ("Living Room", org_id),
            ("Bedroom", org_id),
            ("Storage Unit", org_id),
            ("Office", org_id),
        ];

        let mut location_ids = Vec::new();
        for (name, org_id) in locations {
            let rec = sqlx::query!(
                "INSERT INTO locations (organization_id, name) VALUES ($1, $2) RETURNING id",
                org_id,
                name
            )
            .fetch_one(self.pool)
            .await?;
            location_ids.push(rec.id);
        }
        println!("  ✓ Created {} locations", location_ids.len());
        Ok(location_ids)
    }

    async fn create_collections(&self, org_id: Uuid) -> Result<Vec<Uuid>> {
        let collections = vec![
            ("Jazz Collection", "All jazz albums", org_id),
            ("Rock Classics", "Classic rock albums", org_id),
            (
                "Reference Books",
                "Technical and reference materials",
                org_id,
            ),
            ("Rare Items", "Collectible and rare pieces", org_id),
        ];

        let mut collection_ids = Vec::new();
        for (name, description, org_id) in collections {
            let rec = sqlx::query!(
                "INSERT INTO collections (organization_id, name, description) VALUES ($1, $2, $3) RETURNING id",
                org_id,
                name,
                description
            )
            .fetch_one(self.pool)
            .await?;
            collection_ids.push(rec.id);
        }
        println!("  ✓ Created {} collections", collection_ids.len());
        Ok(collection_ids)
    }

    async fn create_tags(&self, org_id: Uuid) -> Result<()> {
        let tags = vec![
            "vintage",
            "rare",
            "mint-condition",
            "signed",
            "limited-edition",
            "favorite",
        ];

        for tag in tags {
            sqlx::query!(
                "INSERT INTO tags (organization_id, name) VALUES ($1, $2)",
                org_id,
                tag
            )
            .execute(self.pool)
            .await?;
        }
        println!("  ✓ Created tags");
        Ok(())
    }

    async fn create_items_for_org(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        collections: &[Uuid],
    ) -> Result<()> {
        // Vinyl records (10 items)
        self.create_vinyl_items(org_id, locations, collections)
            .await?;

        // CDs (10 items)
        self.create_cd_items(org_id, locations, collections).await?;

        // Cassettes (8 items)
        self.create_cassette_items(org_id, locations, collections)
            .await?;

        // Books (8 items)
        self.create_book_items(org_id, locations, collections)
            .await?;

        // Scores (6 items)
        self.create_score_items(org_id, locations, collections)
            .await?;

        // Electronics (4 items)
        self.create_electronics_items(org_id, locations, collections)
            .await?;

        // Misc (4 items)
        self.create_misc_items(org_id, locations, collections)
            .await?;

        Ok(())
    }

    async fn create_vinyl_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        collections: &[Uuid],
    ) -> Result<()> {
        let vinyl_data = vec![
            (
                "Kind of Blue - Miles Davis",
                "12_inch",
                "33",
                "stereo",
                1,
                "near_mint",
                "excellent",
                "current",
            ),
            (
                "Abbey Road - The Beatles",
                "12_inch",
                "33",
                "stereo",
                1,
                "excellent",
                "good",
                "current",
            ),
            (
                "Dark Side of the Moon - Pink Floyd",
                "12_inch",
                "33",
                "stereo",
                1,
                "mint",
                "mint",
                "current",
            ),
            (
                "Thriller - Michael Jackson",
                "12_inch",
                "33",
                "stereo",
                1,
                "good",
                "fair",
                "loaned",
            ),
            (
                "Rumours - Fleetwood Mac",
                "12_inch",
                "33",
                "stereo",
                1,
                "excellent",
                "excellent",
                "current",
            ),
            (
                "Led Zeppelin IV",
                "12_inch",
                "33",
                "stereo",
                1,
                "near_mint",
                "good",
                "current",
            ),
            (
                "The Velvet Underground & Nico",
                "12_inch",
                "33",
                "mono",
                1,
                "fair",
                "poor",
                "missing",
            ),
            (
                "Pet Sounds - Beach Boys",
                "12_inch",
                "33",
                "stereo",
                1,
                "excellent",
                "near_mint",
                "current",
            ),
            (
                "Blue Train - John Coltrane",
                "12_inch",
                "45",
                "mono",
                1,
                "good",
                "good",
                "current",
            ),
            (
                "Greatest Hits - Various",
                "6_inch",
                "45",
                "mono",
                1,
                "fair",
                "fair",
                "disposed",
            ),
        ];

        for (idx, (name, size, speed, channels, disks, media_grading, sleeve_grading, state)) in
            vinyl_data.iter().enumerate()
        {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "vinyl",
                    state,
                    name,
                    &format!("A classic vinyl record: {}", name),
                    location_id,
                )
                .await?;

            // Add vinyl details
            sqlx::query(
                "INSERT INTO vinyl_details (item_id, size, speed, channels, disks, media_grading, sleeve_grading)
                 VALUES ($1, $2::vinyl_size, $3::vinyl_speed, $4::vinyl_channels, $5, $6::grading, $7::grading)"
            )
            .bind(item_id)
            .bind(*size)
            .bind(*speed)
            .bind(*channels)
            .bind(*disks)
            .bind(*media_grading)
            .bind(*sleeve_grading)
            .execute(self.pool)
            .await?;

            // Add to collection
            if idx % 3 == 0 && !collections.is_empty() {
                self.add_item_to_collection(item_id, collections[idx % collections.len()])
                    .await?;
            }

            // Add state details
            match *state {
                "loaned" => {
                    sqlx::query!(
                        "INSERT INTO item_loan_details (item_id, date_loaned, loaned_to) VALUES ($1, CURRENT_DATE - INTERVAL '10 days', $2)",
                        item_id,
                        "John Doe"
                    )
                    .execute(self.pool)
                    .await?;
                }
                "missing" => {
                    sqlx::query!(
                        "INSERT INTO item_missing_details (item_id, date_missing) VALUES ($1, CURRENT_DATE - INTERVAL '30 days')",
                        item_id
                    )
                    .execute(self.pool)
                    .await?;
                }
                "disposed" => {
                    sqlx::query!(
                        "INSERT INTO item_disposed_details (item_id, date_disposed) VALUES ($1, CURRENT_DATE - INTERVAL '60 days')",
                        item_id
                    )
                    .execute(self.pool)
                    .await?;
                }
                _ => {}
            }

            // Add some tags
            if idx % 2 == 0 {
                self.add_tag_to_item(item_id, org_id, "vintage").await?;
            }
            if idx == 0 || idx == 2 {
                self.add_tag_to_item(item_id, org_id, "favorite").await?;
            }
        }

        println!("  ✓ Created 10 vinyl items");
        Ok(())
    }

    async fn create_cd_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        collections: &[Uuid],
    ) -> Result<()> {
        let cd_data = vec![
            ("OK Computer - Radiohead", 1, "current"),
            ("The Joshua Tree - U2", 1, "current"),
            ("Nevermind - Nirvana", 1, "current"),
            ("Back in Black - AC/DC", 1, "loaned"),
            ("The Wall - Pink Floyd", 2, "current"),
            ("Greatest Hits Vol 1-3 - Queen", 3, "current"),
            ("Anthology - The Beatles", 3, "current"),
            ("1989 - Taylor Swift", 1, "current"),
            ("Random Access Memories - Daft Punk", 1, "missing"),
            ("The Eminem Show", 1, "current"),
        ];

        for (idx, (name, disks, state)) in cd_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "cd",
                    state,
                    name,
                    &format!("CD: {}", name),
                    location_id,
                )
                .await?;

            sqlx::query!(
                "INSERT INTO cd_details (item_id, disks) VALUES ($1, $2)",
                item_id,
                *disks
            )
            .execute(self.pool)
            .await?;

            if idx % 2 == 0 && !collections.is_empty() {
                self.add_item_to_collection(item_id, collections[0]).await?;
            }

            match *state {
                "loaned" => {
                    sqlx::query!(
                        "INSERT INTO item_loan_details (item_id, date_loaned, date_due_back, loaned_to) VALUES ($1, CURRENT_DATE - INTERVAL '5 days', CURRENT_DATE + INTERVAL '5 days', $2)",
                        item_id,
                        "Jane Smith"
                    )
                    .execute(self.pool)
                    .await?;
                }
                "missing" => {
                    sqlx::query!(
                        "INSERT INTO item_missing_details (item_id, date_missing) VALUES ($1, CURRENT_DATE - INTERVAL '15 days')",
                        item_id
                    )
                    .execute(self.pool)
                    .await?;
                }
                _ => {}
            }
        }

        println!("  ✓ Created 10 CD items");
        Ok(())
    }

    async fn create_cassette_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        _collections: &[Uuid],
    ) -> Result<()> {
        let cassette_data = vec![
            ("Appetite for Destruction - Guns N' Roses", 1, "current"),
            ("Purple Rain - Prince", 1, "current"),
            ("Born in the U.S.A. - Bruce Springsteen", 1, "current"),
            ("The Chronic - Dr. Dre", 1, "current"),
            ("Ten - Pearl Jam", 1, "disposed"),
            ("Graceland - Paul Simon", 1, "current"),
            ("Synchronicity - The Police", 1, "current"),
            ("Like a Prayer - Madonna", 1, "current"),
        ];

        for (idx, (name, cassettes, state)) in cassette_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "cassette",
                    state,
                    name,
                    &format!("Cassette tape: {}", name),
                    location_id,
                )
                .await?;

            sqlx::query!(
                "INSERT INTO cassette_details (item_id, cassettes) VALUES ($1, $2)",
                item_id,
                *cassettes
            )
            .execute(self.pool)
            .await?;

            if *state == "disposed" {
                sqlx::query!(
                    "INSERT INTO item_disposed_details (item_id, date_disposed) VALUES ($1, CURRENT_DATE - INTERVAL '90 days')",
                    item_id
                )
                .execute(self.pool)
                .await?;
            }
        }

        println!("  ✓ Created 8 cassette items");
        Ok(())
    }

    async fn create_book_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        collections: &[Uuid],
    ) -> Result<()> {
        let book_data = vec![
            ("The Lord of the Rings - J.R.R. Tolkien", "current"),
            ("1984 - George Orwell", "current"),
            ("To Kill a Mockingbird - Harper Lee", "loaned"),
            ("The Great Gatsby - F. Scott Fitzgerald", "current"),
            ("Dune - Frank Herbert", "current"),
            ("Foundation - Isaac Asimov", "current"),
            ("The Catcher in the Rye - J.D. Salinger", "current"),
            ("Brave New World - Aldous Huxley", "current"),
        ];

        for (idx, (name, state)) in book_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "book",
                    state,
                    name,
                    &format!("Book: {}", name),
                    location_id,
                )
                .await?;

            if idx % 3 == 0 && !collections.is_empty() {
                self.add_item_to_collection(item_id, collections[2]).await?;
            }

            if *state == "loaned" {
                sqlx::query!(
                    "INSERT INTO item_loan_details (item_id, date_loaned, loaned_to) VALUES ($1, CURRENT_DATE - INTERVAL '20 days', $2)",
                    item_id,
                    "Library Friend"
                )
                .execute(self.pool)
                .await?;
            }

            if idx < 2 {
                self.add_tag_to_item(item_id, org_id, "signed").await?;
            }
        }

        println!("  ✓ Created 8 book items");
        Ok(())
    }

    async fn create_score_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        _collections: &[Uuid],
    ) -> Result<()> {
        let score_data = vec![
            ("Beethoven - Symphony No. 9", "current"),
            ("Bach - Well-Tempered Clavier", "current"),
            ("Mozart - Requiem", "current"),
            ("Chopin - Nocturnes", "current"),
            ("Debussy - Clair de Lune", "current"),
            ("Rachmaninoff - Piano Concerto No. 2", "current"),
        ];

        for (idx, (name, state)) in score_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "score",
                    state,
                    name,
                    &format!("Musical score: {}", name),
                    location_id,
                )
                .await?;

            if idx == 0 {
                self.add_tag_to_item(item_id, org_id, "rare").await?;
            }
        }

        println!("  ✓ Created 6 score items");
        Ok(())
    }

    async fn create_electronics_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        _collections: &[Uuid],
    ) -> Result<()> {
        let electronics_data = vec![
            ("Technics SL-1200 Turntable", "current"),
            ("Sony Walkman WM-D6C", "current"),
            ("Pioneer Elite VSX-LX504 Receiver", "current"),
            ("Nakamichi Dragon Cassette Deck", "disposed"),
        ];

        for (idx, (name, state)) in electronics_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "electronics",
                    state,
                    name,
                    &format!("Electronics: {}", name),
                    location_id,
                )
                .await?;

            if *state == "disposed" {
                sqlx::query!(
                    "INSERT INTO item_disposed_details (item_id, date_disposed) VALUES ($1, CURRENT_DATE - INTERVAL '120 days')",
                    item_id
                )
                .execute(self.pool)
                .await?;
            }

            if idx < 2 {
                self.add_tag_to_item(item_id, org_id, "vintage").await?;
            }
        }

        println!("  ✓ Created 4 electronics items");
        Ok(())
    }

    async fn create_misc_items(
        &self,
        org_id: Uuid,
        locations: &[Uuid],
        collections: &[Uuid],
    ) -> Result<()> {
        let misc_data = vec![
            ("Concert Poster - Woodstock 1969", "current"),
            ("Signed Band T-Shirt - Metallica", "current"),
            ("Record Storage Crate - Vintage", "current"),
            ("Music Magazine Collection 1980s", "current"),
        ];

        for (idx, (name, state)) in misc_data.iter().enumerate() {
            let location_id = locations[idx % locations.len()];
            let item_id = self
                .create_item(
                    org_id,
                    "misc",
                    state,
                    name,
                    &format!("Miscellaneous item: {}", name),
                    location_id,
                )
                .await?;

            if idx == 1 {
                self.add_tag_to_item(item_id, org_id, "signed").await?;
                self.add_tag_to_item(item_id, org_id, "rare").await?;
            }

            if !collections.is_empty() {
                self.add_item_to_collection(item_id, collections[3]).await?;
            }
        }

        println!("  ✓ Created 4 misc items");
        Ok(())
    }

    async fn create_item(
        &self,
        org_id: Uuid,
        item_type: &str,
        state: &str,
        name: &str,
        description: &str,
        location_id: Uuid,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            r#"INSERT INTO items
            (organization_id, item_type, state, name, description, location_id, date_acquired)
            VALUES ($1, $2::item_type, $3::item_state, $4, $5, $6, CURRENT_DATE - (random() * 365)::int)
            RETURNING id"#
        )
        .bind(org_id)
        .bind(item_type)
        .bind(state)
        .bind(name)
        .bind(description)
        .bind(location_id)
        .fetch_one(self.pool)
        .await?;

        let id: Uuid = row.get("id");
        Ok(id)
    }

    async fn add_item_to_collection(&self, item_id: Uuid, collection_id: Uuid) -> Result<()> {
        sqlx::query!(
            "INSERT INTO item_collections (item_id, collection_id) VALUES ($1, $2)",
            item_id,
            collection_id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn add_tag_to_item(&self, item_id: Uuid, org_id: Uuid, tag_name: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO item_tags (item_id, organization_id, tag_name) VALUES ($1, $2, $3)",
            item_id,
            org_id,
            tag_name
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
