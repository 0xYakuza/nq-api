// @generated automatically by Diesel CLI.

diesel::table! {
    app_accounts (id) {
        id -> Int4,
        uuid -> Uuid,
        username -> Varchar,
        account_type -> Text,
    }
}

diesel::table! {
    app_emails (id) {
        id -> Int4,
        account_id -> Int4,
        email -> Text,
        verified -> Bool,
        primary -> Bool,
        deleted -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_employees (id) {
        id -> Int4,
        org_account_id -> Int4,
        employee_account_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_organization_names (id) {
        id -> Int4,
        uuid -> Uuid,
        account_id -> Int4,
        name -> Varchar,
        language -> Varchar,
    }
}

diesel::table! {
    app_organizations (id) {
        id -> Int4,
        account_id -> Int4,
        owner_account_id -> Int4,
        profile_image -> Nullable<Text>,
        established_date -> Date,
        national_id -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_permission_conditions (id) {
        id -> Int4,
        permission_id -> Int4,
        name -> Varchar,
        value -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_permissions (id) {
        id -> Int4,
        subject -> Varchar,
        object -> Varchar,
        action -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_tokens (id) {
        id -> Int4,
        account_id -> Int4,
        token_hash -> Varchar,
        terminated -> Bool,
        terminated_by_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_user_names (id) {
        id -> Int4,
        account_id -> Int4,
        primary_name -> Bool,
        first_name -> Varchar,
        last_name -> Varchar,
        language -> Varchar,
    }
}

diesel::table! {
    app_users (id) {
        id -> Int4,
        account_id -> Int4,
        birthday -> Nullable<Date>,
        profile_image -> Nullable<Text>,
        language -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    app_verify_codes (id) {
        id -> Int4,
        code -> Int4,
        email -> Text,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    casbin_rule (id) {
        id -> Int4,
        ptype -> Varchar,
        v0 -> Varchar,
        v1 -> Varchar,
        v2 -> Varchar,
        v3 -> Varchar,
        v4 -> Varchar,
        v5 -> Varchar,
    }
}

diesel::table! {
    mushafs (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    quran_ayahs (id) {
        id -> Int4,
        uuid -> Uuid,
        surah_id -> Int4,
        ayah_number -> Int4,
        sajdeh -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    quran_surahs (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Varchar,
        period -> Nullable<Varchar>,
        number -> Int4,
        bismillah_status -> Varchar,
        bismillah_text -> Nullable<Text>,
        mushaf_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    quran_words (id) {
        id -> Int4,
        uuid -> Uuid,
        ayah_id -> Int4,
        word -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    translations (id) {
        id -> Int4,
        uuid -> Nullable<Uuid>,
        translator_id -> Int4,
        language -> Varchar,
        release_year -> Nullable<Date>,
        source -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    translations_text (id) {
        id -> Int4,
        uuid -> Nullable<Uuid>,
        translation_id -> Int4,
        ayah_id -> Int4,
        text -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(app_emails -> app_accounts (account_id));
diesel::joinable!(app_organization_names -> app_accounts (account_id));
diesel::joinable!(app_organizations -> app_accounts (account_id));
diesel::joinable!(app_permission_conditions -> app_permissions (permission_id));
diesel::joinable!(app_tokens -> app_accounts (account_id));
diesel::joinable!(app_user_names -> app_accounts (account_id));
diesel::joinable!(app_users -> app_accounts (account_id));
diesel::joinable!(quran_ayahs -> quran_surahs (surah_id));
diesel::joinable!(quran_surahs -> mushafs (mushaf_id));
diesel::joinable!(quran_words -> quran_ayahs (ayah_id));
diesel::joinable!(translations_text -> quran_ayahs (ayah_id));
diesel::joinable!(translations_text -> translations (translation_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_accounts,
    app_emails,
    app_employees,
    app_organization_names,
    app_organizations,
    app_permission_conditions,
    app_permissions,
    app_tokens,
    app_user_names,
    app_users,
    app_verify_codes,
    casbin_rule,
    mushafs,
    quran_ayahs,
    quran_surahs,
    quran_words,
    translations,
    translations_text,
);
