-- Relocate the SYSTEM organization from all-zeros to all-ones.
--
-- The SYSTEM org was seeded at 00000000-0000-0000-0000-000000000000, which is exactly
-- Uuid::nil() -- the same value used for an unauthenticated AuthContext's organization_id.
-- That collision makes a bare org-id comparison treat an unauthenticated caller as a
-- SYSTEM member. Moving the SYSTEM org to ffffffff-ffff-ffff-ffff-ffffffffffff keeps
-- Uuid::nil() as an unambiguous "no org" sentinel.
--
-- Tables reference organizations(id) with ON DELETE CASCADE but no ON UPDATE CASCADE, so
-- we insert the new row, repoint every child reference, then delete the old row.

INSERT INTO organizations (id, name, description)
VALUES (
    'ffffffff-ffff-ffff-ffff-ffffffffffff',
    'SYSTEM',
    'System organization for administrative data'
);

UPDATE user_organizations SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE locations SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE collections SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE tags SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE item_tags SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE items SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE audit_log SET organization_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE organization_id = '00000000-0000-0000-0000-000000000000';
UPDATE kinds SET org_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE org_id = '00000000-0000-0000-0000-000000000000';
UPDATE fields SET org_id = 'ffffffff-ffff-ffff-ffff-ffffffffffff'
    WHERE org_id = '00000000-0000-0000-0000-000000000000';

DELETE FROM organizations WHERE id = '00000000-0000-0000-0000-000000000000';
