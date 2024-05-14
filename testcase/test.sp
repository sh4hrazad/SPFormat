#define DEBUG 1

#pragma newdecls required
#pragma semicolon 1

#include <sdktools>
#include <sdkhooks>
#include <zStore>
#include <convar_class>
#include <saikyohopz-cp>
#include <saikyohopz-msg>
#include <clientprefs>
#include <skokit>

public Plugin myinfo = {
    name = SKO_PLUGIN_PREFIX ... "Character Pairs",
    author = "Shahrazad",
    description = "character pairs stuff replicated from zombieden",
    url = "mufiu.com",
};

#define MAX_CP_LEVEL 3
#define FAVOR_INCREMENT_INTERVAL 60
#define GOLD_INTERVAL 900
#define MENU_ITEM_NEW_LINE "\n    "
#define BUTTERFLY_DIST 50.0

enum {
    Cp_Type_Partner = 0,
    Cp_Type_Foes,
    Cp_Type_Lover,
    // Cp_Type_Mentoring,
    Cp_Type_Count,
    Cp_Breakup = Cp_Type_Count,
};

enum {
    Confirm_Seek = 's',
    Confirm_Accept = 'a',
    Confirm_Deny = 'd',
    Confirm_Delay = 'p',
    Confirm_Upgrade = 'u',
    Confirm_Breakup = 'b',
};

enum {
    Sprite_Partner = 0,
    Sprite_Love,
    Sprite_Overhead_Count,
};

enum {
    Sprite_Butterfly = 0,
    Sprite_Butterfly_Color,
    Sprite_Butterfly_Count,
};

enum {
    Materials_Count = Sprite_Overhead_Count + Sprite_Butterfly_Count,
};

enum {
    Butterfly_Color_Yellow = 0,
    Butterfly_Color_Red,
    Butterfly_Color_Green,
    Butterfly_Color_Blue,
    Butterfly_Color_Count,

    Butterfly_Color_F = Butterfly_Color_Count,
};

enum {
    Sound_Establish = 0,
    Sound_Upgrade,
    Sound_Breakup,
    Sound_LoverDeath,
    Sound_InTeam, // ?
    Sound_AskFor,
    Sound_JoinGame,
    Sound_Count,
};

enum struct cp_invitation_t {
    int m_iSenderClient;
    int m_iType;

    void Reset() {
        this.m_iSenderClient = 0;
        this.m_iType = 0;
    }
}

        typeset CpQueryCallback {
// call another query
function void (int client, CpQueryCallback callback, any data);

function void (int client, any data);

function void (int client);
}

// database columns:
// cp_id, s_id, m_id, type, favorability, level, establish_time
Database g_hSql = null;
bool g_bLate;
bool g_bReadOnlySql;

int g_iPlayerId                 [MAXPLAYERS + 1];
int g_iMenuType                 [MAXPLAYERS + 1];
int g_iSendingInvitation        [MAXPLAYERS + 1];
bool g_bDelayConfirm            [MAXPLAYERS + 1];
bool g_bDisableEffects          [MAXPLAYERS + 1];
Menu g_hAcceptCpConfirmMenu     [MAXPLAYERS + 1];
cp_invitation_t g_aInvitation   [MAXPLAYERS + 1];
int g_iButterflies              [MAXPLAYERS + 1];
bool g_bIsButterflyFollowing    [MAXPLAYERS + 1];

int g_iCpId                     [MAXPLAYERS + 1][Cp_Type_Count];
int g_iAllOnlineTime            [MAXPLAYERS + 1][Cp_Type_Count];
bool g_bIsClientSender          [MAXPLAYERS + 1][Cp_Type_Count];
character_pairs_t g_aCpsInGame  [MAXPLAYERS + 1][Cp_Type_Count];

int g_iSprites                  [MAXPLAYERS + 1][Sprite_Overhead_Count];
char g_sCpInfoMenuTitleCache    [MAXPLAYERS + 1][128];

Convar g_hMinPlayersToGainOnlineGoldsConvar;
int g_iMinPlayersToGainOnlineGolds = 10;

GlobalForward g_hForwardOnNewCp;
Cookie g_hDisableEffectsCookie;

static char g_sVtfFilePaths[Materials_Count][PLATFORM_MAX_PATH] = {
        "materials/saikyohopz/cp/partner.vtf",
        "materials/saikyohopz/cp/love.vtf",
        "materials/zombieden/xmode/yuyuko/butter/butter.vtf",
        "materials/zombieden/xmode/yuyuko/butter/butter_color.vtf",
};

static char g_sVmtFilePaths[Materials_Count][PLATFORM_MAX_PATH] = {
        "materials/saikyohopz/cp/partner.vmt",
        "materials/saikyohopz/cp/love.vmt",
        "materials/zombieden/xmode/yuyuko/butter/butter.vmt",
        "materials/zombieden/xmode/yuyuko/butter/butter_color.vmt",
};

static char g_sSoundFilePaths[Sound_Count][PLATFORM_MAX_PATH] = {
        "sound/saikyohopz/cp/halo_es.mp3",
        "sound/saikyohopz/cp/star_up.mp3",
        "sound/saikyohopz/cp/unoh_bu.mp3",
        "sound/saikyohopz/cp/boom.mp3",
        "sound/saikyohopz/cp/inteam.mp3",
        "sound/saikyohopz/cp/askfor.wav",
        "sound/saikyohopz/cp/join.wav",
};

static char g_sSoundFilePaths2[Sound_Count][PLATFORM_MAX_PATH] = {
        "saikyohopz/cp/halo_es.mp3",
        "saikyohopz/cp/star_up.mp3",
        "saikyohopz/cp/unoh_bu.mp3",
        "saikyohopz/cp/boom.mp3",
        "saikyohopz/cp/inteam.mp3",
        "saikyohopz/cp/askfor.wav",
        "saikyohopz/cp/join.wav",
};

#define BUTTERFLY_MODELS_COUNT 2

static char g_sButterflyMdlPaths[BUTTERFLY_MODELS_COUNT][] = {
        "models/zombieden/xmode/yuyuko/butter.mdl",
        "models/zombieden/xmode/yuyuko/butter2.mdl",
};

static char g_sExtraButterflyMdlPaths[BUTTERFLY_MODELS_COUNT * 5][] = {
        "models/zombieden/xmode/yuyuko/butter.dx80.vtx",
        "models/zombieden/xmode/yuyuko/butter.dx90.vtx",
        "models/zombieden/xmode/yuyuko/butter.phy",
        "models/zombieden/xmode/yuyuko/butter.sw.vtx",
        "models/zombieden/xmode/yuyuko/butter.vvd",
        "models/zombieden/xmode/yuyuko/butter2.dx80.vtx",
        "models/zombieden/xmode/yuyuko/butter2.dx90.vtx",
        "models/zombieden/xmode/yuyuko/butter2.phy",
        "models/zombieden/xmode/yuyuko/butter2.sw.vtx",
        "models/zombieden/xmode/yuyuko/butter2.vvd",
};

static char g_sButterflyColors[Butterfly_Color_Count + 1][] = {
        "255 255 0",
        "255 0 0",
        "0 255 0",
        "0 0 255",

        // :)
        "255 106 106",
};

static int g_iUpgradeGolds[Cp_Type_Count][MAX_CP_LEVEL + 1] = {
        { 520, 300, 520, 99999 },
        { 520, 80, 99999, 99999 },
        { 520, 520, 520, 1314 },
        // { 450, 9999, 99999, 99999 },
};

static int g_iUpgradeFavors[Cp_Type_Count][MAX_CP_LEVEL + 1] = {
        { 0, 440, 3500, 99999 },
        { 0, 300, 99999, 99999 },
        { 0, 520, 5200, 5200 },
        // { 0, 99999, 99999, 99999 },
};

static char g_sCpColor[Cp_Type_Count + 1][] = {
        "{blue}",
        "{green}",
        "{hotpink}",
        // "{greenyellow}",
        "{black}",
};

static char g_sCpTypeCn[Cp_Type_Count][] = { "基友", "损友", "情侣"/*, "师徒"*/ };

static char g_sTips[][] = {
        "与你的伙伴一起游戏 (非观察者) 可增加好感度.",
        "当好感度达到一定值后, 可用金币进行升级获取相应的效果.",
};

static char g_sCpSkillTips[Cp_Type_Count][MAX_CP_LEVEL][] = {
        {
                "站在一起的时候将会拥有肥皂泡特效.", // ok
                "可以带给你更多的金币, 双方存活且在同一个阵营, 每隔一段时间会获得金币.", // to be tested
                "暂无效果..",
        },
        {
                "一方失去金币的时候, 另一方将获得等量的金币." ... MENU_ITEM_NEW_LINE ...
        "*赠送金币等非正常金币支出途径将不会获得技能效果.", // to be tested
        "暂无效果..",
        "暂无效果..",
        },
        {
                "站在一起的时候将会拥有爱心特效", // ok
                "可以带给你更多的金币, 双方存活时, 一方获得金币, 另一方也会获得等量的金币.", // ok
                "伴侣阵亡后会化成【灵蝶】跟随你.", // ok
        },
        // {
        //         "暂无效果..",
        //         "暂无效果..",
        //         "暂无效果..",
        // },
};

/* -- Entrypoints -- */

public APLRes AskPluginLoad2(Handle myself, bool late, char[] error, int err_max) {
    CreateNative("Saikyohopz_GetCpStruct", Native_GetCpStruct);

    RegPluginLibrary("saikyohopz-cp");

    g_bLate = late;

    return APLRes_Success;
}

public void OnPluginStart() {
    ConnectToDatabase();

    g_hMinPlayersToGainOnlineGoldsConvar = new Convar(
            "scp_min_players_for_online_golds", "10",
            "Minimum players for partners to gain golds when playing together.",
            0, true, 0.0
    );

    g_hMinPlayersToGainOnlineGoldsConvar.AddChangeHook(OnConvarChanged);

    Convar.AutoExecConfig();

    RegConsoleCmd("sm_zcp", Command_CpMenu, "shows character pairs menu (zed alias)");
    RegConsoleCmd("sm_scp", Command_CpMenu, "shows character pairs menu");
    RegConsoleCmd("sm_zcpe", Command_ToggleEffects, "toggles cp effects (zed alias)");
    RegConsoleCmd("sm_scpe", Command_ToggleEffects, "toggles cp effects");

    RegAdminCmd("sm_updatecp", Command_UpdateCp, ADMFLAG_GENERIC, "updates all online players' cp info (!!! server may be stuck on calling it)");
    RegAdminCmd("sm_dumpcp", Command_Debug_DumpCp, ADMFLAG_GENERIC, "(debug) dumps your own cp info");

    HookEvent("player_spawn", Event_PlayerSpawn, EventHookMode_Pre);
    HookEvent("player_death", Event_PlayerDeath, EventHookMode_Pre);

    g_hForwardOnNewCp = new GlobalForward("Saikyohopz_OnNewCp", ET_Ignore, Param_Cell, Param_Cell, Param_Cell);
    g_hDisableEffectsCookie = new Cookie("saikyohopz_cp_disable_effects", "[Saikyohopz CP] Disable Effects", CookieAccess_Protected);

    if (!g_bLate) {
        return;
    }

    for (int i = 1; i <= MaxClients; i++) {
        if (IsValidClient(i)) {
            // OnClientPutInServer(i);

            if (AreClientCookiesCached(i)) {
                OnClientCookiesCached(i);
            }
        }
    }
}

public void Saikyohopz_OnChatLoadedPost() {
    Saikyohopz_RegisterChatTag("cpair", Tag_Cp);
}

public void OnPluginEnd() {
    Saikyohopz_UnregisterChatTag("cpair");
}

public void OnMapStart() {
    g_bReadOnlySql = false;

    LoadContents();

    CreateTimer(1.0, Timer_Cron, 0, TIMER_FLAG_NO_MAPCHANGE | TIMER_REPEAT);
}

public void ZS_OnPlayerLoad(int client, int id)
{
    g_iPlayerId[client] = id;

    FetchCps(client, PrintJoinMessage);
}

/*
public void OnClientPutInServer(int client) {
        if (IsFakeClient(client)) {
                return;
        }

        g_iPlayerId[client] = ZS_GetPlayerID(client);

        FetchCps(client, PrintJoinMessage);
}
*/

public void OnClientCookiesCached(int client) {
    char cookie[8];

    GetClientCookie(client, g_hDisableEffectsCookie, cookie, sizeof(cookie));

    if (strlen(cookie) == 0) {
        SetCookieBool(client, g_hDisableEffectsCookie, false);
    }

    g_bDisableEffects[client] = GetCookieBool(client, g_hDisableEffectsCookie);
}

public void OnClientDisconnect(int client) {
    if (IsFakeClient(client)) {
        return;
    }

    PrintQuitMessage(client);

    // cancel invitation
    AbortAcceptCpConfirmation(client);

    g_iSendingInvitation[client] = 0;

    UpdateCps(client);

    for (int i = 0; i < Cp_Type_Count; i++) {
        ResetCpInfo(client, i, false);
    }

    g_bDelayConfirm[client] = false;
    g_hAcceptCpConfirmMenu[client] = null;
    g_iPlayerId[client] = 0;
}

public void OnPlayerGoldAmountChanged(int client, int target, int golds, int reason) {
    int foes = ZS_GetClientFromId(g_iCpId[client][Cp_Type_Foes]);

    if (IsValidClient(foes) && g_aCpsInGame[client][Cp_Type_Foes].m_iLevel >= 1) {
        if (reason == GoldReason_Backstab && golds < 0) {
            ZS_GivePlayerGold(foes, -golds, client, GoldReason_FoesBonus);
        }
    }

    int lover = ZS_GetClientFromId(g_iCpId[client][Cp_Type_Lover]);

    if (IsValidClient(lover) && g_aCpsInGame[client][Cp_Type_Lover].m_iLevel >= 2) {
        if (golds > 0 && (
                reason == GoldReason_FinishMap ||
                reason == GoldReason_ReachCheckpoint ||
                reason == GoldReason_GoldEntity ||
                reason == GoldReason_WinGame
        )) {
            ZS_GivePlayerGold(lover, golds, client, GoldReason_LoverBonus);
        }
    }
}

public Action OnPlayerRunCmd(
        int client, int &buttons, int &impulse,
        float vel[3], float angles[3], int &weapon,
        int &subtype, int &cmdnum, int &tickcount,
        int &seed, int mouse[2]
) {
    DrawOverheadSprite(client);
    ProceedButterflyMovement(client);

    return Plugin_Continue;
}

void LoadContents() {
    for (int i = 0; i < Materials_Count; i++) {
        AddFileToDownloadsTable(g_sVtfFilePaths[i]);
        AddFileToDownloadsTable(g_sVmtFilePaths[i]);

        if (!PrecacheModel(g_sVmtFilePaths[i])) {
            LogError("Failed to precache .vmt file: %s", g_sVmtFilePaths[i]);
        }
    }

    for (int i = 0; i < BUTTERFLY_MODELS_COUNT; i++) {
        AddFileToDownloadsTable(g_sButterflyMdlPaths[i]);

        // if (!PrecacheModel(g_sButterflyMdlPaths[i], true)) {
        //         LogError("Failed to precache .mdl file: %s", g_sButterflyMdlPaths[i]);
        // }
    }

    for (int i = 0; i < BUTTERFLY_MODELS_COUNT * 5; i++) {
        AddFileToDownloadsTable(g_sExtraButterflyMdlPaths[i]);
    }

    for (int i = 0; i < Sound_Count; i++) {
        AddFileToDownloadsTable(g_sSoundFilePaths[i]);

        if (!PrecacheSound(g_sSoundFilePaths2[i], true)) {
            LogError("Failed to precache sound file: %s", g_sSoundFilePaths2[i]);
        }
    }
}

int Native_GetCpStruct(Handle plugin, int numParams) {
    int client = GetNativeCell(1);
    int type = GetNativeCell(2);

    character_pairs_t cp;

    CopyCpStruct(g_aCpsInGame[client][type], cp);

    SetNativeArray(3, cp, sizeof(character_pairs_t));

    return 1;
}

Action Timer_Cron(Handle timer) {
    int clients = GetClientCount(true);

    for (int i = 1; i <= MaxClients; i++) {
        if (!IsValidClient(i, true)) {
            continue;
        }

        for (int j = 0; j < Cp_Type_Count; j++) {
            if (
                    !IsValidClient(ZS_GetClientFromId(g_iCpId[i][j]), true)
                    ) {
                continue;
            }

            g_iAllOnlineTime[i][j]++;

            if (g_iAllOnlineTime[i][j] % FAVOR_INCREMENT_INTERVAL == 0) {
                g_aCpsInGame[i][j].m_iFavorability++;
            }
        }

        if (clients < g_iMinPlayersToGainOnlineGolds) {
            continue;
        }

        if (
                g_aCpsInGame[i][Cp_Type_Partner].m_iLevel >= 2 &&
                g_iAllOnlineTime[i][Cp_Type_Partner] % GOLD_INTERVAL == 0
                ) {
            ZS_GivePlayerGold(i, 1, 0, GoldReason_PartnerBonus);
        }
    }

    return Plugin_Continue;
}

void Tag_Cp(int client, const char[] tagName, char[] buffer, int buflen) {
    // TODO: Cp name
}

void OnConvarChanged(ConVar cvar, const char[] oldValue, const char[] newValue) {
    char cvarName[64];

    cvar.GetName(cvarName, sizeof(cvarName));

    if (StrEqual(cvarName, "scp_min_players_for_online_golds")) {
        g_iMinPlayersToGainOnlineGolds = StringToInt(newValue);
    }
}

Action Command_CpMenu(int client, int args) {
    if (client == 0) {
        return Plugin_Handled;
    }

    OpenMainMenu(client);

    return Plugin_Handled;
}

Action Command_ToggleEffects(int client, int args) {
    if (client == 0) {
        return Plugin_Handled;
    }

    g_bDisableEffects[client] = !g_bDisableEffects[client];
    SetCookieBool(client, g_hDisableEffectsCookie, g_bDisableEffects[client]);

    CPRINT(client, "{white}CP 特效已%s{white}.", g_bDisableEffects[client] ? "{red}禁用" : "{lightgreen}启用");

    return Plugin_Handled;
}

Action Command_UpdateCp(int client, int args) {
    for (int i = 1; i <= MaxClients; i++) {
        if (IsValidClient(i)) {
            UpdateCps(i, FetchCps);
        }
    }

    PrintToServer("Updated CP info for all online players.");

    if (client != 0) {
        CPRINT(client, "{white}已{lightgreen}更新{white}所有在线玩家的 CP 数据.");
    }

    return Plugin_Handled;
}

Action Command_Debug_DumpCp(int client, int args) {
#if DEBUG
    if (client == 0) {
        return Plugin_Handled;
    }

    CPRINT(client, "self player id: %d", g_iPlayerId[client]);

    CPRINT(client, "id senderid sendername accepter acceptername type favorabilty level establishtime");

    for (int i = 0; i < Cp_Type_Count; i++) {
        CPRINT(client, "%d %d %s %d %s %d %d %d %d",
               g_aCpsInGame[client][i].m_iCpId,
               g_aCpsInGame[client][i].m_iSenderId,
               g_aCpsInGame[client][i].m_sSenderName,
               g_aCpsInGame[client][i].m_iAccepterId,
               g_aCpsInGame[client][i].m_sAccepterName,
               g_aCpsInGame[client][i].m_iType,
               g_aCpsInGame[client][i].m_iFavorability,
               g_aCpsInGame[client][i].m_iLevel,
               g_aCpsInGame[client][i].m_iEstablishTime
        );
    }
#endif

    return Plugin_Handled;
}

void OpenMainMenu(int client) {
    Menu menu = new Menu(MenuHandler_MainMenu);

    menu.SetTitle("- Saikyohopz Character Pairs\n- !scpe - 开启/关闭特效%s\n ", g_bReadOnlySql ? "\nWARNING: DATABASE IS NOW READ ONLY!" : "");

    char display[192];
    char name[MAX_NAME_LENGTH];

    for (int i = 0; i < Cp_Type_Count; i++) {
        if (HasCp(client, i)) {
            GetCpName(client, i, name, sizeof(name));

            Format(
                    display, sizeof(display), "%s: %s%s- 好感度: %d | Lv. %d\n ", g_sCpTypeCn[i],
                    name, MENU_ITEM_NEW_LINE,
                    g_aCpsInGame[client][i].m_iFavorability,
                    g_aCpsInGame[client][i].m_iLevel
            );
        } else {
            Format(display, sizeof(display), "%s: 暂无..%s- N/A\n ", g_sCpTypeCn[i], MENU_ITEM_NEW_LINE);
        }

        char info[2];

        IntToString(i, info, sizeof(info));

        menu.AddItem(info, display);
    }

    Format(display, sizeof(display), "- 提示:");

    for (int i = 0; i < sizeof(g_sTips); i++) {
        Format(display, sizeof(display), "%s%s%d. %s", display, MENU_ITEM_NEW_LINE, i + 1, g_sTips[i]);
    }

    menu.AddItem("", display, ITEMDRAW_DISABLED);

    menu.Pagination = MENU_NO_PAGINATION;
    menu.ExitButton = true;

    menu.Display(client, MENU_TIME_FOREVER);
}

int MenuHandler_MainMenu(Menu menu, MenuAction action, int param1, int param2) {
    switch (action) {
        case MenuAction_Select: {
            char info[2];

            int dummy;

            menu.GetItem(
                    param2, info, sizeof(info), dummy,
                    g_sCpInfoMenuTitleCache[param1],
                    sizeof(g_sCpInfoMenuTitleCache[])
            );

            ReplaceString(
                    g_sCpInfoMenuTitleCache[param1],
                    sizeof(g_sCpInfoMenuTitleCache[]),
                    "    ", "", false
            );

            g_iMenuType[param1] = StringToInt(info);

            OpenCpInfoMenu(param1);
        }

        case MenuAction_End: {
            delete menu;
        }
    }

    return 0;
}

void OpenCpInfoMenu(int client) {
    int type = g_iMenuType[client];

    Menu menu = new Menu(MenuHandler_CpInfoMenu);

    menu.SetTitle(g_sCpInfoMenuTitleCache[client]);

    char display[384];

    if (HasCp(client, type) && !g_bReadOnlySql) {
        int level = g_aCpsInGame[client][type].m_iLevel;

        if (level < MAX_CP_LEVEL) {
            int favor = g_aCpsInGame[client][type].m_iFavorability;
            int requiredFavor = g_iUpgradeFavors[type][level] - favor;

            Format(display, sizeof(display), "升级 -> Lv. %d%s- ", level + 1, MENU_ITEM_NEW_LINE);

            int itemDraw = ITEMDRAW_DEFAULT;

            if (requiredFavor > 0) {
                Format(display, sizeof(display), "%s暂不可用, 仍需 %d 好感度\n ", display, requiredFavor);

                itemDraw = ITEMDRAW_DISABLED;
            } else {
                int requiredGolds = g_iUpgradeGolds[type][level];

                Format(display, sizeof(display), "%s需要 %d 金币\n ", display, requiredGolds);

                if (ZS_GetPlayerGold(client) < requiredGolds) {
                    itemDraw = ITEMDRAW_DISABLED;
                }
            }

            menu.AddItem("upgrade", display, itemDraw);
        }

        Format(display, sizeof(display), "查看当前在线的%s", g_sCpTypeCn[type]);

        menu.AddItem("list", display);

        Format(display, sizeof(display), "解除当前%s关系\n ", g_sCpTypeCn[type]);

        menu.AddItem("breakup", display);
    } else if (!g_bReadOnlySql) {
        Format(
                display, sizeof(display), "向一名玩家发送%s请求%s- 需要 %d 金币\n ",
                g_sCpTypeCn[type], MENU_ITEM_NEW_LINE, g_iUpgradeGolds[type][0]
        );

        menu.AddItem("list", display);
    }

    Format(display, sizeof(display), "解锁效果:");

    for (int i = 0; i < MAX_CP_LEVEL; i++) {
        Format(display, sizeof(display), "%s%s- Lv. %d: %s\n", display, MENU_ITEM_NEW_LINE, i + 1, g_sCpSkillTips[type][i]);
    }

    menu.AddItem("", display, ITEMDRAW_DISABLED);

    menu.Pagination = MENU_NO_PAGINATION;
    menu.ExitButton = true;

    menu.Display(client, MENU_TIME_FOREVER);
}

int MenuHandler_CpInfoMenu(Menu menu, MenuAction action, int param1, int param2) {
    switch (action) {
        case MenuAction_Select: {
            char info[8];

            menu.GetItem(param2, info, sizeof(info));

            if (StrEqual(info, "upgrade")) {
                OpenConfirmMenu(param1, Confirm_Upgrade);
            } else if (StrEqual(info, "list")) {
                OpenListMenu(param1);
            } else if (StrEqual(info, "breakup")) {
                OpenBreakupConfirmMenu(param1);
            }
        }

        case MenuAction_Cancel: {
            OpenMainMenu(param1);
        }

        case MenuAction_End: {
            delete menu;
        }
    }

    return 0;
}

void OpenListMenu(int client) {
    int type = g_iMenuType[client];

    bool seekMode = !HasCp(client, type);

    Menu menu = new Menu(MenuHandler_ListMenu);

    if (seekMode) {
        menu.SetTitle("向一名玩家发送%s请求\n ", g_sCpTypeCn[type]);
    } else {
        menu.SetTitle("当前在线的%s:\n ", g_sCpTypeCn[type]);
    }

    char display[128];
    int hasCp[MAXPLAYERS + 1];

    for (int i = 1; i <= MaxClients; i++) {
        if (!IsValidClient(i)) {
            continue;
        }

        int itemDraw = ITEMDRAW_DISABLED;
        bool isValidItem;

        if (HasCp(i, type)) {
            int cpTarget = ZS_GetClientFromId(g_iCpId[i][type]);

            // ignore duplicate info
            if (hasCp[i] == cpTarget) {
                continue;
            }

            if (cpTarget != -1) {
                hasCp[cpTarget] = i;
            }

            Format(
                    display, sizeof(display),
                    "%s <-> %s%s- 好感度: %d | Lv. %d\n ",
                    g_aCpsInGame[i][type].m_sSenderName,
                    g_aCpsInGame[i][type].m_sAccepterName,
                    MENU_ITEM_NEW_LINE,
                    g_aCpsInGame[i][type].m_iFavorability,
                    g_aCpsInGame[i][type].m_iLevel
            );

            isValidItem = true;
        } else if (seekMode) {
            if (i == client || IsFakeClient(i)) {
                continue;
            }

            char name[MAX_NAME_LENGTH];

            GetClientName(i, name, sizeof(name));

            Format(display, sizeof(display), "%s%s- 没有%s", name, MENU_ITEM_NEW_LINE, g_sCpTypeCn[type]);

            if (!g_iSendingInvitation[client]) {
                itemDraw = ITEMDRAW_DEFAULT;
            }

            isValidItem = true;
        }

        if (isValidItem) {
            char info[3];

            IntToString(i, info, sizeof(info));

            menu.AddItem(info, display, itemDraw);
        }
    }

    if (menu.ItemCount == 0) {
        menu.AddItem("", "暂无结果...", ITEMDRAW_DISABLED);
    }

    menu.ExitButton = true;

    menu.Display(client, MENU_TIME_FOREVER);
}

int MenuHandler_ListMenu(Menu menu, MenuAction action, int param1, int param2) {
    switch (action) {
        case MenuAction_Select: {
            char info[3];

            menu.GetItem(param2, info, sizeof(info));

            OpenConfirmMenu(param1, Confirm_Seek, StringToInt(info));
        }

        case MenuAction_Cancel: {
            OpenCpInfoMenu(param1);
        }

        case MenuAction_End: {
            delete menu;
        }
    }

    return 0;
}

void OpenAcceptCpConfirmMenu(int client, int sender, bool noDelayCheck = false) {
    Menu menu = new Menu(MenuHandler_ConfirmMenu);

    char name[MAX_NAME_LENGTH];

    GetClientName(sender, name, sizeof(name));

    menu.SetTitle("玩家 [%s] 希望与你结成%s关系, 是否同意?\n ", name, g_sCpTypeCn[g_iMenuType[sender]]);

    int itemDraw = ITEMDRAW_DEFAULT;

    if (!noDelayCheck) {
        g_bDelayConfirm[client] = IsClientInGame(client) && !IsFakeClient(client) && GetClientMenu(client) != MenuSource_None;

        if (g_bDelayConfirm[client]) {
            CPRINT(client, "{white}为了防止误操作, 你需要等待 {lightgreen}1.75s{white} 后才能确认.");

            DataPack dp = new DataPack();

            dp.WriteCell(GetClientSerial(client));
            dp.WriteCell(sender);

            CreateTimer(1.85, Timer_DelayConfirm, dp, TIMER_FLAG_NO_MAPCHANGE);

            itemDraw = ITEMDRAW_DISABLED;
        }
    }

    char info[4];

    Format(
            info, sizeof(info), "%d%c", sender,
            (g_bDelayConfirm[client] && !noDelayCheck) ? Confirm_Delay : Confirm_Accept
    );

    PlaySoundCp(client, false, Sound_AskFor);

    menu.AddItem(info, "同意!!!!!", itemDraw);

    Format(info, sizeof(info), "%d%c", sender, Confirm_Deny);

    menu.AddItem(info, "拒绝", itemDraw);

    menu.Pagination = MENU_NO_PAGINATION;
    menu.ExitButton = false;

    menu.Display(client, MENU_TIME_FOREVER);

    g_hAcceptCpConfirmMenu[client] = menu;
}

Action Timer_DelayConfirm(Handle timer, DataPack data) {
    data.Reset();

    int client = GetClientFromSerial(data.ReadCell());
    int sender = data.ReadCell();

    delete data;

    if (g_bDelayConfirm[client]) {
        if (IsClientInGame(client)) {
            OpenAcceptCpConfirmMenu(client, sender, true);
        }

        g_bDelayConfirm[client] = false;
    }

    return Plugin_Stop;
}

void AbortAcceptCpConfirmation(int client) {
    if (!g_aInvitation[client].m_iSenderClient) {
        return;
    }

    int accepter = g_iSendingInvitation[client];

    g_hAcceptCpConfirmMenu[accepter].Cancel();
    g_hAcceptCpConfirmMenu[accepter] = null;

    CPRINT(accepter, "{white}请求发送者已离开游戏, 请求已自动{red}取消{white}.");

    g_aInvitation[client].Reset();
}

void OpenBreakupConfirmMenu(int client) {
    Menu menu = new Menu(MenuHandler_ConfirmMenu);

    int type = g_iMenuType[client];

    char name[MAX_NAME_LENGTH];

    GetCpName(client, type, name, sizeof(name));

    menu.SetTitle(
            "!!! 此操作将解除你与 [%s] 的%s关系, 是否确认?\n ",
            name, g_sCpTypeCn[type]
    );

    int yesPos = GetRandomInt(0, 4);

    for (int i = 0; i < 5; i++) {
        if (i != yesPos) {
            menu.AddItem("", "取消");

            continue;
        }

        char info[8];

        Format(
                info, sizeof(info), "%d%c",
                g_aCpsInGame[client][type].m_iCpId, Confirm_Breakup
        );

        menu.AddItem(info, "确认");
    }

    menu.Pagination = MENU_NO_PAGINATION;
    menu.ExitButton = false;

    menu.Display(client, MENU_TIME_FOREVER);
}

void OpenConfirmMenu(int client, int confirmType, any data = 0) {
    Menu menu = new Menu(MenuHandler_ConfirmMenu);

    char confirmInfo[8];

    int cpType = g_iMenuType[client];

    char name[MAX_NAME_LENGTH];

    int nextLevel;

    switch (confirmType) {
        case Confirm_Accept: {
            // see OpenAcceptCpRequestMenu()
        }

        case Confirm_Breakup: {
            // see OpenBreakupConfirmMenu()
        }

        case Confirm_Seek: {
            nextLevel = 0;

            GetClientName(data, name, sizeof(name));

            menu.SetTitle(
                    "将发送与玩家 [%s] 建立%s关系的请求, 是否确认?\n" ...
            "若你在请求超时前离开游戏或对方拒绝, 金币将不会返还.\n ",
                    name, g_sCpTypeCn[cpType]
            );
        }

        case Confirm_Upgrade: {
            nextLevel = g_aCpsInGame[client][cpType].m_iLevel + 1;

            GetCpName(client, cpType, name, sizeof(name));

            menu.SetTitle(
                    "即将提升你与 [%s] 的%s关系等级至 Lv. %d, 是否确认?\n ", name,
                    g_sCpTypeCn[cpType], nextLevel
            );
        }
    }

    Format(confirmInfo, sizeof(confirmInfo), "%d%c", data, confirmType);

    char display[128];

    Format(display, sizeof(display), "确认%s- 花费 %d 金币", MENU_ITEM_NEW_LINE, g_iUpgradeGolds[cpType][nextLevel]);

    if (nextLevel > 0) {
        Format(display, sizeof(display), "%s并获得效果:%s%s", display, MENU_ITEM_NEW_LINE, g_sCpSkillTips[cpType][nextLevel]);
    }

    menu.AddItem(confirmInfo, display);
    menu.AddItem("", "取消");

    menu.Pagination = MENU_NO_PAGINATION;
    menu.ExitButton = false;

    menu.Display(client, MENU_TIME_FOREVER);
}

int MenuHandler_ConfirmMenu(Menu menu, MenuAction action, int param1, int param2) {
    switch (action) {
        case MenuAction_Select: {
            char info[8];

            menu.GetItem(param2, info, sizeof(info));

            if (strlen(info) == 0) {
                OpenCpInfoMenu(param1);

                return 0;
            }

            DoConfirm(param1, StringToInt(info), info[strlen(info) - 1]);
        }

        case MenuAction_Cancel: {
            if (g_bDelayConfirm[param1]) {
                return 0;
            }

            int sender = g_aInvitation[param1].m_iSenderClient;

            ZS_GivePlayerGold(sender, g_iUpgradeGolds[g_aInvitation[param1].m_iType][0], 0, GoldReason_OutOfDate);

            char name[MAX_NAME_LENGTH];

            GetClientName(param1, name, sizeof(name));

            CPRINT(sender, "{white}[%s] 的确认菜单显示{red}超时{white}或被其他菜单{red}覆盖{white}, 请重试.", name);

            g_iSendingInvitation[sender] = 0;
            g_hAcceptCpConfirmMenu[param1] = null;
            g_aInvitation[param1].Reset();
        }

        case MenuAction_End: {
            delete menu;
        }
    }

    return 0;
}

void DoConfirm(int client, int data, char type) {
    if (g_bReadOnlySql) {
        CPRINT(client, "{white}数据库暂不可用, 请稍后再试.");

        return;
    }

    switch (type) {
        case Confirm_Accept: {
            EstablishCp(client);

            g_iSendingInvitation[g_aInvitation[client].m_iSenderClient] = 0;
        }

        case Confirm_Deny: {
            int sender = data;

            if (!IsValidClient(sender)) {
                return;
            }

            char name[MAX_NAME_LENGTH];

            GetClientName(client, name, sizeof(name));

            CPRINT(sender, "{white}[%s] {red}拒绝{white}了你的请求.", name);

            g_iSendingInvitation[sender] = 0;

            g_aInvitation[client].Reset();
        }

        case Confirm_Breakup: {
            int cpId = data;

            BreakupCp(client, cpId);
        }

        case Confirm_Delay: {
            // do nothing...
        }

        case Confirm_Seek: {
            int target = data;

            if (g_iSendingInvitation[client]) {
                CPRINT(client, "{white}你发送的请求还未得到回应, 请稍后再试.");

                return;
            }

            int cpType = g_iMenuType[client];

            if (!IsValidClient(target)) {
                CPRINT(client, "{white}该玩家已{red}离线{white}, 请等待该玩家再次上线后重试.");

                return;
            }

            if (!ZS_GivePlayerGold(client, -g_iUpgradeGolds[cpType][0], 0, GoldReason_BuyItem)) {
                return;
            }

            g_aInvitation[target].m_iSenderClient = client;
            g_aInvitation[target].m_iType = cpType;

            OpenAcceptCpConfirmMenu(target, client);

            CPRINT(client, "{white}请等待该玩家确认.");

            g_iSendingInvitation[client] = target;
        }

        case Confirm_Upgrade: {
            int cpId = data;

            UpgradeCpLevel(client, cpId);
        }
    }
}

/* -- SQL Queries -- */

void ConnectToDatabase() {
    if (g_hSql) {
        delete g_hSql;
    }

    char error[MAX_MESSAGE_LENGTH];

    if (!SQL_CheckConfig("zStore")) {
        SetFailState("Database is not configured in databases.cfg.");
    }

    if (!(g_hSql = SQL_Connect("zStore", true, error, sizeof(error)))) {
        SetFailState("SQL Connection failed. Reason: %s", error);
    }

    if (!g_hSql.SetCharset("utf8mb4")) {
        g_hSql.SetCharset("utf8");
    }

    CreateDbTables();
}

void CreateDbTables() {
    Transaction trans = new Transaction();

    trans.AddQuery(
            "create table if not exists `cps` ( \
                        `cp_id` int not null primary key auto_increment, \
                        `s_id` int not null, \
                        `m_id` int not null, \
                        `type` tinyint not null, \
                        `favorability` int not null default 0, \
                        `level` tinyint not null default 0, \
                        `establish_time` int not null \
                );"
    );

    g_hSql.Execute(trans, INVALID_FUNCTION, CreateDbTables_OnError, 0, DBPrio_High);
}

void CreateDbTables_OnError(
        Database db, any data, int numQueries,
        const char[] error, int failIndex, any[] queryData
) {
    SetFailState("Failed to create tables for the plugin. Reason: %s", error);
}

// insert
void EstablishCp(int client) {
    char query[256];

    Format(
            query, sizeof(query), "insert into cps ( \
                        s_id, m_id, type, establish_time \
                ) values (%d, %d, %d, %d);",
            g_iPlayerId[g_aInvitation[client].m_iSenderClient],
            g_iPlayerId[client],
            g_aInvitation[client].m_iType,
            GetTime()
    );

    g_hSql.Query(EstablishCp_Callback, query, GetClientSerial(client));
}

void EstablishCp_Callback(Database db, DBResultSet results, const char[] error, int data) {
    int accepter = GetClientFromSerial(data);
    int sender = g_aInvitation[accepter].m_iSenderClient;
    int type = g_aInvitation[accepter].m_iType;

    g_aInvitation[accepter].Reset();

    if (!results) {
        ZS_GivePlayerGold(sender, g_iUpgradeGolds[type][0], 0, GoldReason_ServerError);

        CPRINT(sender, "{red}数据库异常{white}, 金币已退回...");
        CPRINT(accepter, "{red}数据库异常{white}, 组建失败...");

        LogError("EstablishCp() insertion query failed. Reason: %s", error);

        return;
    }

    FetchCps(accepter);

    char accepterName[MAX_NAME_LENGTH];
    char senderName[MAX_NAME_LENGTH];

    GetClientName(accepter, accepterName, sizeof(accepterName));
    GetClientName(sender, senderName, sizeof(senderName));

    CPRINT(
            accepter, "%s♥{dimgray} - {white}你接受了 [%s%s{white}] 的%s%s{white}请求~!",
            g_sCpColor[type], g_sCpColor[type], senderName, g_sCpColor[type], g_sCpTypeCn[type]
    );

    CPRINT(
            sender, "%s♥{dimgray} - {white}[%s%s{white}] 接受了你的%s%s{white}请求~!",
            g_sCpColor[type], g_sCpColor[type], accepterName, g_sCpColor[type], g_sCpTypeCn[type]
    );

    // anounce it!
    CPRINTALL(
            "%s♥{dimgray} - {white}[%s%s{white}(攻) & %s%s{white}(受)] 恭喜这对%s%s{white}产生~!",
            g_sCpColor[type], g_sCpColor[type], senderName, g_sCpColor[type], accepterName, g_sCpColor[type], g_sCpTypeCn[type]
    );

    PlaySoundCp(sender, true, Sound_Establish, type);

    Call_StartForward(g_hForwardOnNewCp);
    Call_PushCell(sender);
    Call_PushCell(accepter);
    Call_PushCell(type);
    Call_Finish();
}

// delete
void BreakupCp(int client, int cpId) {
    char query[128];

    Format(query, sizeof(query), "delete from cps where cp_id = %d", cpId);

    g_hSql.Query(BreakupCp_Callback, query, GetClientSerial(client));
}

void BreakupCp_Callback(Database db, DBResultSet results, const char[] error, int data) {
    int client = GetClientFromSerial(data);

    if (!results) {
        CPRINT(client, "{red}数据库异常{white}, 解除失败...");

        LogError("BreakupCp() deletion query failed. Reason: %s", error);

        return;
    }

    int type = g_iMenuType[client];
    int anotherClient = ZS_GetClientFromId(g_iCpId[client][type]);
    char name[MAX_NAME_LENGTH];

    GetCpName(client, type, name, sizeof(name));

    ResetCpInfo(client, type, true);

    CPRINT(
            client, "%s♥{dimgray} - {white}你{red}解除{white}了与 [%s] 的%s关系.",
            g_sCpColor[Cp_Breakup], name, g_sCpTypeCn[type]
    );

    if (anotherClient > 0) {
        CPRINT(
                anotherClient, "%s♥{dimgray} - {white}[%N] {red}解除{white}了与你的%s关系.",
                g_sCpColor[Cp_Breakup], client, g_sCpTypeCn[type]
        );

        PlaySoundCp(anotherClient, false, Sound_Breakup, type);
    }

    // announce it (?)

    PlaySoundCp(client, false, Sound_Breakup, type);
}

void UpgradeCpLevel(int client, int cpId) {
    int type = g_iMenuType[client];

    int currentLevel = g_aCpsInGame[client][type].m_iLevel;

    if (!ZS_GivePlayerGold(client, -g_iUpgradeGolds[type][currentLevel], GoldReason_BuyItem)) {
        return;
    }

    UpdateCps(client, __UpgradeCpLevel, cpId);
}

// alter
void __UpgradeCpLevel(int client, any cpId) {
    char query[128];

    Format(query, sizeof(query), "update cps set level = level + 1 where cp_id = %d", cpId);

    g_hSql.Query(UpgradeCpLevel_Callback, query, GetClientSerial(client));
}

void UpgradeCpLevel_Callback(Database db, DBResultSet results, const char[] error, int data) {
    int client = GetClientFromSerial(data);

    if (!results) {
        CPRINT(client, "{red}数据库异常{white}, 升级失败...");

        LogError("UpgradeCpLevel() update query failed. Reason: %s", error);

        return;
    }

    FetchCps(client, UpgradeCpLevel_SemiCallback);
}

void UpgradeCpLevel_SemiCallback(int client) {
    int type = g_iMenuType[client];

    int currentLevel = g_aCpsInGame[client][type].m_iLevel;

    char name[MAX_NAME_LENGTH];

    GetCpName(client, type, name, sizeof(name));

    CPRINT(
            client, "%s♥{dimgray} - {white}你与 [%s%s{white}] 的%s关系已升级至 Lv. %d.",
            g_sCpColor[type], g_sCpColor[type], name, g_sCpTypeCn[type], currentLevel
    );

    int anotherClient = ZS_GetClientFromId(g_iCpId[client][type]);

    if (anotherClient > 0) {
        GetClientName(client, name, sizeof(name));

        CPRINT(
                anotherClient, "%s♥{dimgray} - {white}你与 [%s%s{white}] 的%s关系已升级至 Lv. %d.",
                g_sCpColor[type], g_sCpColor[type], name, g_sCpTypeCn[type], currentLevel
        );
    }

    PlaySoundCp(client, false, Sound_Upgrade, type);
}

// select
void FetchCps(int client, CpQueryCallback callback = INVALID_FUNCTION, any data = 0) {
    char query[320];

    StrCat(
            query, sizeof(query), "select cp_id, s_id, m_id, \
                (select nick from info where id = s_id) as s_name, \
                (select nick from info where id = m_id) as m_name, \
                type, favorability, level, establish_time from cps where"
    );

    int id = g_iPlayerId[client];

    Format(query, sizeof(query), "%s s_id = %d or m_id = %d;", query, id, id);

    DataPack dp = new DataPack();

    dp.WriteFunction(callback);
    dp.WriteCell(client);
    dp.WriteCell(data);

    g_hSql.Query(FetchCps_Callback, query, dp);
}

void FetchCps_Callback(Database db, DBResultSet results, const char[] error, DataPack dp) {
    if (!results) {
        LogError("FetchCps() selection query failed. Reason: %s", error);

        return;
    }

    dp.Reset();

    Function func = dp.ReadFunction();
    int client = dp.ReadCell();
    any data = dp.ReadCell();

    delete dp;

    while (results.FetchRow()) {
        /**
         * 0 - cp_id
         * 1 - s_id
         * 2 - m_id
         * 3 - s_name
         * 4 - m_name
         * 5 - type
         * 6 - favorability
         * 7 - level
         * 8 - establish_time
         */

        int type = results.FetchInt(5);

        if (!IsValidCpType(type)) {
            continue;
        }

        int sId = results.FetchInt(1);
        int mId = results.FetchInt(2);

        int sClient = ZS_GetClientFromId(sId);
        int mClient = ZS_GetClientFromId(mId);

#if DEBUG
        PrintToServer("fetched cp client index (playerid): %d, %d (%d, %d)", sClient, mClient, sId, mId);
#endif

        character_pairs_t cp;

        cp.m_iCpId = results.FetchInt(0);
        cp.m_iSenderId = sId;
        cp.m_iAccepterId = mId;
        results.FetchString(3, cp.m_sSenderName, sizeof(character_pairs_t::m_sSenderName));
        results.FetchString(4, cp.m_sAccepterName, sizeof(character_pairs_t::m_sAccepterName));
        cp.m_iType = type;
        cp.m_iFavorability = results.FetchInt(6);
        cp.m_iLevel = results.FetchInt(7);
        cp.m_iEstablishTime = results.FetchInt(8);

        if (IsValidClient(sClient)) {
            ResetCpInfo(sClient, type, false);
            ApplyCpInfo(sClient, cp);
        }

        if (IsValidClient(mClient)) {
            ResetCpInfo(mClient, type, false);
            ApplyCpInfo(mClient, cp);
        }
    }

    if (func == INVALID_FUNCTION) {
        return;
    }

    Call_StartFunction(INVALID_HANDLE, func);
    Call_PushCell(client);
    Call_PushCell(data);
    Call_Finish();
}

void UpdateCps(int client, CpQueryCallback callback = INVALID_FUNCTION, any data = 0) {
    if (g_bReadOnlySql && callback != INVALID_FUNCTION) {
        Call_StartFunction(INVALID_HANDLE, callback);
        Call_PushCell(client);
        Call_PushCell(data);
        Call_Finish();

        return;
    }

    Transaction trans = new Transaction();

    char query[256];

    for (int i = 0; i < Cp_Type_Count; i++) {
        if (!g_aCpsInGame[client][i].m_iCpId) {
            continue;
        }

        int structClient = client;

        if (!g_bIsClientSender[structClient]) {
            structClient = ZS_GetClientFromId(GetAnotherIdFromCpInfo(client, i));

            if (!IsValidClient(structClient)) {
                continue;
            }
        }

        Format(
                query, sizeof(query), "update cps set favorability = %d where cp_id = %d;",
                g_aCpsInGame[structClient][i].m_iFavorability, g_aCpsInGame[structClient][i].m_iCpId
        );

        trans.AddQuery(query);
    }

    DataPack dp = new DataPack();

    dp.WriteFunction(callback);
    dp.WriteCell(client);
    dp.WriteCell(data);

    g_hSql.Execute(trans, UpdateCps_Callback, INVALID_FUNCTION, dp);
}

void UpdateCps_Callback(Database db, DataPack dp, int numQueries, DBResultSet[] results, any[] queryData) {
    dp.Reset();

    Function func = dp.ReadFunction();
    int client = dp.ReadCell();
    any data = dp.ReadCell();

    delete dp;

    if (func == INVALID_FUNCTION) {
        return;
    }

    Call_StartFunction(INVALID_HANDLE, func);
    Call_PushCell(client);
    Call_PushCell(data);
    Call_Finish();
}

bool IsValidClient(int client, bool alive = false) {
    return (
            client >= 1 &&
            client <= MaxClients &&
            IsClientConnected(client) &&
            IsClientInGame(client) &&
            !IsClientSourceTV(client) &&
            (!alive || IsPlayerAlive(client))
    );
}

bool HasCp(int client, int type) {
    return g_aCpsInGame[client][type].m_iCpId > 0;
}

void GetCpName(int client, int type, char[] buffer, int buflen) {
    strcopy(
            buffer, buflen,
            g_bIsClientSender[client][type] ?
            g_aCpsInGame[client][type].m_sAccepterName :
            g_aCpsInGame[client][type].m_sSenderName
    );
}

int GetAnotherIdFromCpInfo(int client, int type) {
    if (!HasCp(client, type)) {
        return -1;
    }

    if (g_aCpsInGame[client][type].m_iSenderId == g_iPlayerId[client]) {
        return g_aCpsInGame[client][type].m_iAccepterId;
    } else if (g_aCpsInGame[client][type].m_iAccepterId == g_iPlayerId[client]) {
        return g_aCpsInGame[client][type].m_iSenderId;
    }

    return -1;
}

void CopyCpStruct(any[] from, any[] to) {
// to.m_iCpId = from.m_iCpId;
// to.m_iSenderId = from.m_iSenderId;
// to.m_iAccepterId = from.m_iAccepterId;
// strcopy(to.m_sSenderName, sizeof(character_pairs_t::m_sSenderName), from.m_sSenderName);
// strcopy(to.m_sAccepterName, sizeof(character_pairs_t::m_sAccepterName), from.m_sAccepterName);
// to.m_iType = from.m_iType;
// to.m_iFavorability = from.m_iFavorability;
// to.m_iLevel = from.m_iLevel;
// to.m_iEstablishTime = from.m_iEstablishTime;

// cleaner
for (int i = 0; i < sizeof(character_pairs_t); i++) {
to[i] = from[i];
}
}

void ResetCpStruct(any[] cp) {
// cp.m_iCpId = 0;
// cp.m_iSenderId = 0;
// cp.m_iAccepterId = 0;
// strcopy(cp.m_sSenderName, sizeof(character_pairs_t::m_sSenderName), "");
// strcopy(cp.m_sAccepterName, sizeof(character_pairs_t::m_sAccepterName), "");
// cp.m_iType = 0;
// cp.m_iFavorability = 0;
// cp.m_iLevel = 0;
// cp.m_iEstablishTime = 0;

// cleaner
for (int i = 0; i < sizeof(character_pairs_t); i++) {
cp[i] = 0;
}
}

bool IsValidCpType(int type) {
    return 0 <= type < Cp_Type_Count;
}

void ApplyCpInfo(int client, character_pairs_t cp) {
    int type = cp.m_iType;

    if (!IsValidClient(client)) {
        return;
    }

    CopyCpStruct(cp, g_aCpsInGame[client][type]);

    g_iCpId[client][type] = GetAnotherIdFromCpInfo(client, type);

    if (g_iPlayerId[client] == g_aCpsInGame[client][type].m_iSenderId) {
        g_bIsClientSender[client][type] = true;
    }
}

void ResetCpInfo(int client, int type, bool resetBoth) {
    int anotherClient = ZS_GetClientFromId(g_iCpId[client][type]);

    Semi_ResetCpInfo(client, type, true);

    if (IsValidClient(anotherClient)) {
        Semi_ResetCpInfo(anotherClient, type, resetBoth);
    }
}

void Semi_ResetCpInfo(int client, int type, bool resetStruct) {
    if (resetStruct) {
        ResetCpStruct(g_aCpsInGame[client][type]);

        g_bIsClientSender[client][type] = false;
    }

    if (type == Cp_Type_Partner) {
        RemoveOverheadSprite(client, Sprite_Partner);
    } else if (type == Cp_Type_Lover) {
        RemoveOverheadSprite(client, Sprite_Love);
        RemoveButterfly(client);
    }

    g_iCpId[client][type] = 0;
    g_iAllOnlineTime[client][type] = 0;
}

void PlaySoundCp(int client, bool everyone, int soundType, int cpType = -1) {
    int clients[MAXPLAYERS + 1];
    int count = 0;

    for (int i = 1; i <= MaxClients; i++) {
        if (IsValidClient(i) && (everyone || i == client || (
                cpType != -1 && i == ZS_GetClientFromId(g_iCpId[client][cpType])
        ))) {
            clients[count++] = i;
        }
    }

    if (count) {
        EmitSound(clients, count, g_sSoundFilePaths2[soundType]);
    }
}

void PrintJoinMessage(int client, any quitMessage = false) {
    char name[MAX_NAME_LENGTH];

    GetClientName(client, name, sizeof(name));

    bool fetchedCp = false;

    for (int i = 0; i < Cp_Type_Count; i++) {
        int anotherClient = ZS_GetClientFromId(g_iCpId[client][i]);

        if (!IsValidClient(anotherClient) || g_aCpsInGame[client][i].m_iCpId == 0) {
            continue;
        }

        LogMessage("[SKO-CP] PrintJoinMessage(): client: %d, anotherClient: %d", client, anotherClient);

        fetchedCp = true;

        CPRINT(
                anotherClient, "%s | %s♥{dimgray} - {white}你的%s [%s%s{white}] 已%s",
                quitMessage ? "{red}▼{white}" : "{green}▲{white}",
                g_sCpColor[i], g_sCpTypeCn[i], g_sCpColor[i], name,
                quitMessage ? "离开游戏." : "上线!"
        );

        if (!quitMessage) {
            char cpName[MAX_NAME_LENGTH];

            GetCpName(client, i, cpName, sizeof(cpName));

            CPRINT(
                    client, "%s♥{dimgray} - {white}你的%s [%s%s{white}] 正在游戏中!",
                    g_sCpColor[i], g_sCpTypeCn[i], g_sCpColor[i], cpName
            );

            PlaySoundCp(anotherClient, false, Sound_JoinGame);
        }
    }

    if (fetchedCp) {
        PlaySoundCp(client, false, Sound_JoinGame);
    }
}

void PrintQuitMessage(int client) {
    PrintJoinMessage(client, true);
}

void CreateOverheadSprite(int client, int spriteType) {
    int envSprite = CreateEntityByName("env_sprite");

    if (!SanityIsValidEntity(envSprite)) {
        return;
    }

    DispatchKeyValue(envSprite, "model", g_sVmtFilePaths[spriteType]);
    DispatchKeyValue(envSprite, "spawnflags", "1");
    DispatchKeyValue(envSprite, "scale", "0.5");
    DispatchKeyValue(envSprite, "rendermode", "1");
    DispatchKeyValue(envSprite, "rendercolor", "255 255 255");
    DispatchKeyValue(envSprite, "framerate", "10");
    DispatchSpawn(envSprite);

    SDKHook(envSprite, SDKHook_SetTransmit, OnEntityTransmit);

    SetEntityFlags(envSprite, 1);

    float spriteOrigin[3];

    GetClientAbsOrigin(client, spriteOrigin);

    spriteOrigin[2] += 96.0;

    TeleportEntity(envSprite, spriteOrigin, NULL_VECTOR, NULL_VECTOR);

    SetVariantString("!activator");

    AcceptEntityInput(envSprite, "SetParent", client);

    g_iSprites[client][spriteType] = envSprite;
}

void SafelyDeleteEntity(int entity) {
    // 0 is valid entity???
    if (SanityIsValidEntity(entity)) {
        AcceptEntityInput(entity, "Kill");
    }
}

void RemoveOverheadSprite(int client, int spriteType) {
    SafelyDeleteEntity(g_iSprites[client][spriteType]);

    g_iSprites[client][spriteType] = 0;
}

int GetOverheadSpriteType(int type) {
    return type / 2;
}

void DrawOverheadSprite(int client) {
    if (g_aCpsInGame[client][Cp_Type_Partner].m_iLevel >= 1) {
        Semi_DrawOverheadSprite(client, Cp_Type_Partner);
    }

    if (g_aCpsInGame[client][Cp_Type_Lover].m_iLevel >= 1) {
        Semi_DrawOverheadSprite(client, Cp_Type_Lover);
    }
}

void Semi_DrawOverheadSprite(int client, int type) {
    int anotherClient = ZS_GetClientFromId(g_iCpId[client][type]);
    int spriteType = GetOverheadSpriteType(type);

    if (!IsPlayerAlive(client) || !IsValidClient(anotherClient, true)) {
        RemoveOverheadSprite(client, spriteType);

        return;
    }

    float pos[2][3];

    GetClientAbsOrigin(client, pos[0]);
    GetClientAbsOrigin(anotherClient, pos[1]);

    float distance = GetVectorDistance(pos[0], pos[1]);

    if (distance > 196.0 && g_iSprites[client][spriteType]) {
        RemoveOverheadSprite(client, spriteType);
    } else if (distance <= 196.0 && !g_iSprites[client][spriteType]) {
        CreateOverheadSprite(client, spriteType);
    }
}

void Event_PlayerDeath(Event event, const char[] name, bool dontBroadcast) {
    int client = GetClientOfUserId(event.GetInt("userid"));

    RemoveButterfly(client);

    if (g_aCpsInGame[client][Cp_Type_Lover].m_iLevel >= 3) {
        CreateButterfly(ZS_GetClientFromId(g_iCpId[client][Cp_Type_Lover]));
    }

    for (int i = 0; i < Sprite_Overhead_Count; i++) {
        RemoveOverheadSprite(client, i);
    }

    // int anotherClient = g_iAnotherClient[client][Cp_Type_Lover];

    // if (IsValidClient(anotherClient)) {
    //         RemoveOverheadSprite(anotherClient, Sprite_Love);
    // }

    // anotherClient = g_iAnotherClient[client][Cp_Type_Partner];

    // if (IsValidClient(anotherClient)) {
    //         RemoveOverheadSprite(anotherClient, Sprite_Partner);
    // }
}

void Event_PlayerSpawn(Event event, const char[] name, bool dontBroadcast) {
    int client = GetClientOfUserId(event.GetInt("userid"));
    int anotherClient = ZS_GetClientFromId(g_iCpId[client][Cp_Type_Lover]);

    if (IsValidClient(anotherClient)) {
        RemoveButterfly(anotherClient);
    }
}

void CreateButterfly(int client) {
    if (!IsValidClient(client, true)) {
        return;
    }

    int entity = CreateEntityByName("prop_dynamic");

    DispatchKeyValue(entity, "model", g_sButterflyMdlPaths[GetRandomInt(0, BUTTERFLY_MODELS_COUNT - 1)]);
    DispatchKeyValue(entity, "rendercolor", g_sButterflyColors[GetRandomInt(0, Butterfly_Color_Count - 1)]);
    DispatchKeyValue(entity, "DefaultAnim", "idle");
    DispatchKeyValue(entity, "solid", "0");
    DispatchSpawn(entity);

    SDKHook(entity, SDKHook_SetTransmit, OnEntityTransmit);

    float origin[3];

    GetClientAbsOrigin(ZS_GetClientFromId(g_iCpId[client][Cp_Type_Lover]), origin);

    origin[2] += 48.0;

    // teleport to the cp who dies
    TeleportEntity(entity, origin, NULL_VECTOR, NULL_VECTOR);

    CreateTimer(1.0, Timer_DelayButterflyMovement, GetClientSerial(client));

    g_iButterflies[client] = entity;

    EmitAmbientSound(g_sSoundFilePaths2[Sound_LoverDeath], origin, SOUND_FROM_WORLD, SNDLEVEL_NORMAL, _);
}

void RemoveButterfly(int client) {
    SafelyDeleteEntity(g_iButterflies[client]);

    g_bIsButterflyFollowing[client] = false;
    g_iButterflies[client] = 0;
}

Action Timer_DelayButterflyMovement(Handle timer, int serial) {
    int client = GetClientFromSerial(serial);

    if (!SanityIsValidEntity(g_iButterflies[client])) {
        return Plugin_Stop;
    }

    g_bIsButterflyFollowing[client] = true;

    return Plugin_Stop;
}

void ProceedButterflyMovement(int client) {
    if (!g_bIsButterflyFollowing[client] || !g_iButterflies[client]) {
        return;
    }

    float clientOrigin[3];
    float butterflyOrigin[3];

    GetClientAbsOrigin(client, clientOrigin);
    GetEntityAbsOrigin(g_iButterflies[client], butterflyOrigin);

    float movingDir[3];

    SubtractVectors(clientOrigin, butterflyOrigin, movingDir);

    float norm = SquareRoot(Pow(movingDir[0], 2.0) + Pow(movingDir[1], 2.0));
    float scale = BUTTERFLY_DIST / norm;
    float desiredDest[3];

    LerpVector(clientOrigin, butterflyOrigin, scale, desiredDest);

    desiredDest[2] = clientOrigin[2] + 72.0;

    float lerpedDest[3];

    LerpVector(butterflyOrigin, desiredDest, 0.08, lerpedDest);

    float movingAngles[3];

    GetVectorAngles(movingDir, movingAngles);

    movingAngles[1] -= 90.0;

    TeleportEntity(g_iButterflies[client], lerpedDest, movingAngles, NULL_VECTOR);

#if DEBUG
    SetHudTextParams(-1.0, 0.8, 0.1, 255, 255, 255, 255, 0, 1.0, 0.0, 0.0);

    ShowHudText(
            client, 0, "%.2f, %.2f, %.2f | %.2f, %.2f, %.2f | %.2f",
            butterflyOrigin[0], butterflyOrigin[1], butterflyOrigin[2],
            lerpedDest[0], lerpedDest[1], lerpedDest[2], scale
    );
#endif
}

float Lerp(float from, float to, float fraction) {
    return from + (to - from) * fraction;
}

void LerpVector(float from[3], float to[3], float fraction, float buffer[3]) {
    for (int i = 0; i < 3; i++) {
        buffer[i] = Lerp(from[i], to[i], fraction);
    }
}

void GetEntityAbsOrigin(int entity, float vec[3]) {
    GetEntPropVector(entity, Prop_Send, "m_vecOrigin", vec);
}

Action OnEntityTransmit(int entity, int client) {
    if (g_bDisableEffects[client]) {
        return Plugin_Handled;
    }

    return Plugin_Continue;
}

bool SanityIsValidEntity(int entity) {
    return entity > 0 && IsValidEntity(entity);
}

void SetCookieBool(int client, Cookie cookie, bool value) {
    if (!AreClientCookiesCached(client)) {
        return;
    }

    char cookieStr[8];

    IntToString(value, cookieStr, sizeof(cookieStr));

    SetClientCookie(client, cookie, cookieStr);
}

bool GetCookieBool(int client, Cookie cookie) {
    if (!AreClientCookiesCached(client)) {
        ThrowError("client's cookie has not cached! client index: %d", client);
    }

    char cookieStr[8];

    GetClientCookie(client, cookie, cookieStr, sizeof(cookieStr));

    if (strlen(cookieStr) == 0) {
        return true;
    }

    return view_as<bool>(StringToInt(cookieStr));
}
