// external/NRI/nri_all.h
#ifndef NRI_ALL_H_
#define NRI_ALL_H_

// 1. Alapvető makrók és leírók (Core API)
#include "_NRI_SDK/Include/NRIMacro.h"
#include "_NRI_SDK/Include/NRIDescs.h"
#include "_NRI_SDK/Include/NRI.h"

// 2. Modern grafikai kiterjesztések (EZ JÖN ELŐRE, hogy a típusok már elérhetőek legyenek)
#include "_NRI_SDK/Include/Extensions/NRIMeshShader.h"
#include "_NRI_SDK/Include/Extensions/NRIRayTracing.h"
#include "_NRI_SDK/Include/Extensions/NRILowLatency.h"
#include "_NRI_SDK/Include/Extensions/NRIStreamer.h"
#include "_NRI_SDK/Include/Extensions/NRIUpscaler.h"

// 3. Alapvető kiterjesztések az ablakkezeléshez és eszközlétrehozáshoz
#include "_NRI_SDK/Include/Extensions/NRIDeviceCreation.h"
#include "_NRI_SDK/Include/Extensions/NRISwapChain.h"
#include "_NRI_SDK/Include/Extensions/NRIHelper.h"

// 4. Linux-specifikus grafikus backend (Vulkan) - Most már ismerni fogja az AccelerationStructure típusokat
#include "_NRI_SDK/Include/Extensions/NRIWrapperVK.h"

// 5. UI kiterjesztés
#include "_NRI_SDK/Include/Extensions/NRIImgui.h"

#endif // NRI_ALL_H_

