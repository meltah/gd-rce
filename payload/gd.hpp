#pragma once
#include "basic.hpp"

struct CCSize {
	float width;
	float height;
};

struct CCPoint {
	float x;
	float y;
};

struct CCObject {
	void** vtable;
	unsigned int m_uID;
	int m_nLuaID;
	int m_nTag;
	unsigned int m_uReference;
	unsigned int m_uAutoReleaseCount;
	int m_eObjType;
	unsigned int m_uObjectIdxInArray;
};

struct CCArray : CCObject {
	void* data;
};

struct _listEntry {
	_listEntry *prev, *next;
	CCObject* target;
	int priority;
	bool paused;
	bool markedForDeletion;
};

struct CCAffineTransform {
  float a, b, c, d;
  float tx, ty;
};

struct CCScheduler {
	u8 pad[0x24];
	_listEntry* m_pUpdatesNegList;
	_listEntry* m_pUpdates0List;
	_listEntry* m_pUpdatesPosList;
};

struct CCNode : CCObject {
	float m_fRotationX;                 ///< rotation angle on x-axis
    float m_fRotationY;                 ///< rotation angle on y-axis
    
    float m_fScaleX;                    ///< scaling factor on x-axis
    float m_fScaleY;                    ///< scaling factor on y-axis
    
    float m_fVertexZ;                   ///< OpenGL real Z vertex
    
    CCPoint m_obPosition;               ///< position of the node
    
    float m_fSkewX;                     ///< skew angle on x-axis
    float m_fSkewY;                     ///< skew angle on y-axis
    
    CCPoint m_obAnchorPointInPoints;    ///< anchor point in points
    CCPoint m_obAnchorPoint;            ///< anchor point normalized (NOT in points)
    
    CCSize m_obContentSize;             ///< untransformed size of the node
    
    
    CCAffineTransform m_sAdditionalTransform; ///< transform
    CCAffineTransform m_sTransform;     ///< transform
    CCAffineTransform m_sInverse;       ///< transform
    
    void *m_pCamera;                ///< a camera
    
    void *m_pGrid;                ///< a grid
    
    int m_nZOrder;                      ///< z-order value that affects the draw order
    
    CCArray *m_pChildren;               ///< array of children nodes
    CCNode *m_pParent;                  ///< weak reference to parent node

    
    void *m_pUserData;                  ///< A user assingned void pointer, Can be point to any cpp object
    CCObject *m_pUserObject;            ///< A user assigned CCObject
    
    void *m_pShaderProgram;      ///< OpenGL shader
    
    int m_eGLServerState;   ///< OpenGL servier side state
    
    unsigned int m_uOrderOfArrival;     ///< used to preserve sequence while sorting children with the same zOrder
    
    CCScheduler *m_pScheduler;          ///< scheduler used to schedule timers and updates
    
    void *m_pActionManager;  ///< a pointer to ActionManager singleton, which is used to handle all the actions
    
    bool m_bRunning;                    ///< is running
    
    bool m_bTransformDirty;             ///< transform dirty flag
    bool m_bInverseDirty;               ///< transform dirty flag
    bool m_bAdditionalTransformDirty;   ///< The flag to check whether the additional transform is dirty
    bool m_bVisible;                    ///< is this node visible
    
    bool m_bIgnoreAnchorPointForPosition; ///< true if the Anchor Point will be (0,0) when you position the CCNode, false otherwise.
                                          ///< Used by CCLayer and CCScene.
    
    bool m_bReorderChildDirty;          ///< children order dirty flag
    
    int m_nScriptHandler;               ///< script handler for onEnter() & onExit(), used in Javascript binding and Lua binding.
    int m_nUpdateScriptHandler;         ///< script handler for update() callback per frame, which is invoked from lua & javascript.
    int m_eScriptType;         ///< type of script binding, lua or javascript
    
    void *m_pComponentContainer;        ///< Dictionary of components
};

struct CCLayer : CCNode {
	u8 pad[0x11c - sizeof(CCNode)];
};

static_assert(sizeof(CCLayer) == 0x11c);

struct CCScene : CCNode {

};

struct CCDirector : CCObject {
	u8 pad[0x28];
	CCScheduler* m_pScheduler;
};

static_assert(sizeof(CCDirector) == 0x4c);

struct PlayLayer;
struct PlayerObject;
struct GJEffectManager;

struct GameManager {
    char _pad[0x164];
    PlayLayer* m_playLayer;
};

struct GameObject : CCNode {
	char _pad[0x340];
	int m_nObjectID;
};

struct PlayerCheckpoint {};

struct CheckpointObject : CCNode {
	char _pad[0xcc];
	GameObject* m_gameObject; // 0x0EC 
	PlayerCheckpoint* m_player1; // 0x0F0 
	PlayerCheckpoint* m_player2; // 0x0F4 
	bool m_isDual; // 0x0F8 
	bool m_isFlipped; // 0x0F9 
	CCPoint m_cameraPos; // 0x0FC unsure
	int unk104; // comes from playlayer + 2ac
	GameObject* m_lastPortal; // 0x108 
	int paddingHint; // padding, but we use it to indicate that this is a modified checkpoint
	double unk110;
	char m_currentStateString[0x18];
	char m_objectsStateString[0x18];
};

struct GJBaseGameLayer {};

struct PlayLayer : CCLayer {
	char _pad[0x8];
	GJEffectManager* m_effectManager;
	CCLayer* m_pObjectLayer;
	char _pad2[0xf8];
	PlayerObject* m_player1;
	PlayerObject* m_player2;
	char _pad3[0xc0];
	bool isMoving;
	char _pad4[0x1b];
	float groundRestriction;
	float ceilRestriction;
	char _pad5[0x21f];
	char isPaused;
};

struct GJEffectManager : CCObject {

};

struct PlayerObject {
	char _pad[0x34];
	CCPoint m_obPosition; // 0x34
	char _pad2[0x608];
	float m_vehicleSize; // 0x644
	float m_playerSpeed; // 0x648
	char _pad3[0x30];
	CCPoint m_position; // 0x67C
};
