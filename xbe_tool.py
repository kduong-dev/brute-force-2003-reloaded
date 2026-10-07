#!/usr/bin/env python3
"""
xbe_tool.py - Original Xbox executable (.xbe) analyzer, aimed at PC-port feasibility.

Usage:
  python xbe_tool.py info    <file.xbe> [--json]   header, certificate, sections, XDK libs, kernel imports
  python xbe_tool.py hwscan  <file.xbe> [-v N]     disassemble; find instructions touching NV2A/APU MMIO
  python xbe_tool.py strings <file.xbe> [-n MIN] [--section NAME]   ASCII strings with virtual addresses
  python xbe_tool.py dump    <file.xbe> <section> <out.bin>         raw section bytes

Layout reference: XBE header at 0x0 ('XBEH'), image base normally 0x00010000.
Entry point and kernel thunk address are XOR-obfuscated with retail or debug keys.
"""
import argparse
import json
import re
import struct
import sys
from collections import Counter, defaultdict

XOR_EP_RETAIL, XOR_EP_DEBUG = 0xA8FC57AB, 0x94859D4B
XOR_KT_RETAIL, XOR_KT_DEBUG = 0x5B6D40B6, 0xEFB1F152

SECTION_FLAGS = {0x1: 'W', 0x2: 'preload', 0x4: 'X', 0x8: 'insertfile', 0x10: 'headpage_ro', 0x20: 'tailpage_ro'}

REGIONS = {0x1: 'NA', 0x2: 'JP', 0x4: 'RestOfWorld', 0x80000000: 'Manufacturing'}
MEDIA = {0x1: 'HDD', 0x2: 'DVD_X2', 0x4: 'DVD_CD', 0x8: 'CD', 0x10: 'DVD5_RO', 0x20: 'DVD9_RO',
         0x40: 'DVD5_RW', 0x80: 'DVD9_RW', 0x100: 'Dongle', 0x200: 'MediaBoard',
         0x40000000: 'NonSecureHDD', 0x80000000: 'NonSecureMode'}

# What each XDK library means for a PC port (keyed on name prefix).
LIB_NOTES = {
    'D3D8':     'Direct3D 8 (Xbox flavour, NV2A push-buffer backend) -> D3D9/11/Vulkan wrapper',
    'D3DX8':    'D3DX helpers (math/mesh/texture) -> mostly portable to PC D3DX/DirectXMath',
    'XGRAPHC':  'XGraphics (texture swizzle, push-buffer helpers) -> unswizzle on load',
    'DSOUND':   'DirectSound (Xbox APU/MCPX backend) -> XAudio2/OpenAL/FAudio',
    'XACTENG':  'XACT audio engine (.xwb/.xsb) -> FAudio has an XACT3-ish implementation',
    'XAPILIB':  'XAPI (Win32 subset: files, threads, controllers, saves) -> Win32 shim',
    'XONLINE':  'Xbox Live client -> stub out',
    'XONLINES': 'Xbox Live client (secure) -> stub out',
    'XNET':     'Xbox network stack (system link) -> Winsock shim or stub',
    'XNETS':    'Xbox network stack (secure) -> Winsock shim or stub',
    'XVOICE':   'Voice chat (communicator) -> stub',
    'XHV':      'Voice chat engine -> stub',
    'XMV':      'Xbox media video playback -> replace',
    'LIBC':     'C runtime -> PC CRT',
    'LIBCMT':   'C runtime -> PC CRT',
    'LIBCP':    'C++ runtime -> PC CRT',
    'LIBCPMT':  'C++ runtime -> PC CRT',
    'XBOXKRNL': 'kernel import library',
    'DOLBY':    'Dolby Digital encoder microcode -> drop',
    'XKBD':     'USB keyboard',
}

# Kernel subsystem prefix -> porting difficulty / meaning
KRNL_GROUPS = [
    ('Nt',  'native file/object/memory API -> Win32 (CreateFile, VirtualAlloc, ...)'),
    ('Io',  'I/O manager (devices, mounting) -> mostly stub / path remap'),
    ('Ke',  'kernel threads, DPCs, timers, events -> Win32 threads/events'),
    ('Kf',  'IRQL raise/lower -> no-op'),
    ('Ex',  'executive pool alloc, locks -> malloc / critical sections'),
    ('Mm',  'physical/contiguous memory (GPU-visible!) -> VirtualAlloc, but watch physical addr use'),
    ('Ps',  'thread creation -> CreateThread'),
    ('Ob',  'object manager -> handle table shim'),
    ('Rtl', 'runtime library (strings, critsec, unicode) -> trivial'),
    ('Hal', 'hardware abstraction (interrupts, SMC, reboot) -> stub; interrupts = hard'),
    ('Av',  'video encoder / display mode -> stub to window/resolution'),
    ('Xbox','Xbox globals (EEPROM, HD key, kernel version, LaunchData) -> fake data'),
    ('Xe',  'XBE image info / section loader -> reimplement'),
    ('Xc',  'crypto (SHA, RC4, signatures) -> library or stub'),
    ('Fsc', 'filesystem cache -> stub'),
    ('Dbg', 'debug output -> OutputDebugString'),
    ('Ldt', 'LDT entries -> stub'),
    ('Phy', 'physical memory -> stub'),
    ('Idex','IDE channel (raw disk) -> avoid'),
    ('Launch', 'title relaunch data -> fake'),
    ('Hd',  'hard-disk partition info -> fake'),
    ('Write', 'port/register writes -> stub'),
    ('Read', 'port/register reads -> stub'),
]

KERNEL_EXPORTS = {
    1:'AvGetSavedDataAddress', 2:'AvSendTVEncoderOption', 3:'AvSetDisplayMode',
    4:'AvSetSavedDataAddress', 5:'DbgBreakPoint', 6:'DbgBreakPointWithStatus',
    7:'DbgLoadImageSymbols', 8:'DbgPrint', 9:'HalReadSMCTrayState', 10:'DbgPrompt',
    11:'DbgUnLoadImageSymbols', 12:'ExAcquireReadWriteLockExclusive',
    13:'ExAcquireReadWriteLockShared', 14:'ExAllocatePool', 15:'ExAllocatePoolWithTag',
    16:'ExEventObjectType', 17:'ExFreePool', 18:'ExInitializeReadWriteLock',
    19:'ExInterlockedAddLargeInteger', 20:'ExInterlockedAddLargeStatistic',
    21:'ExInterlockedCompareExchange64', 22:'ExMutantObjectType', 23:'ExQueryPoolBlockSize',
    24:'ExQueryNonVolatileSetting', 25:'ExReadWriteRefurbInfo', 26:'ExRaiseException',
    27:'ExRaiseStatus', 28:'ExReleaseReadWriteLock', 29:'ExSaveNonVolatileSetting',
    30:'ExSemaphoreObjectType', 31:'ExTimerObjectType', 32:'ExfInterlockedInsertHeadList',
    33:'ExfInterlockedInsertTailList', 34:'ExfInterlockedRemoveHeadList', 35:'FscGetCacheSize',
    36:'FscInvalidateIdleBlocks', 37:'FscSetCacheSize', 38:'HalClearSoftwareInterrupt',
    39:'HalDisableSystemInterrupt', 40:'HalDiskCachePartitionCount', 41:'HalDiskModelNumber',
    42:'HalDiskSerialNumber', 43:'HalEnableSystemInterrupt', 44:'HalGetInterruptVector',
    45:'HalReadSMBusValue', 46:'HalReadWritePCISpace', 47:'HalRegisterShutdownNotification',
    48:'HalRequestSoftwareInterrupt', 49:'HalReturnToFirmware', 50:'HalWriteSMBusValue',
    51:'InterlockedCompareExchange', 52:'InterlockedDecrement', 53:'InterlockedIncrement',
    54:'InterlockedExchange', 55:'InterlockedExchangeAdd', 56:'InterlockedFlushSList',
    57:'InterlockedPopEntrySList', 58:'InterlockedPushEntrySList', 59:'IoAllocateIrp',
    60:'IoBuildAsynchronousFsdRequest', 61:'IoBuildDeviceIoControlRequest',
    62:'IoBuildSynchronousFsdRequest', 63:'IoCheckShareAccess', 64:'IoCompletionObjectType',
    65:'IoCreateDevice', 66:'IoCreateFile', 67:'IoCreateSymbolicLink', 68:'IoDeleteDevice',
    69:'IoDeleteSymbolicLink', 70:'IoDeviceObjectType', 71:'IoFileObjectType', 72:'IoFreeIrp',
    73:'IoInitializeIrp', 74:'IoInvalidDeviceRequest', 75:'IoQueryFileInformation',
    76:'IoQueryVolumeInformation', 77:'IoQueueThreadIrp', 78:'IoRemoveShareAccess',
    79:'IoSetIoCompletion', 80:'IoSetShareAccess', 81:'IoStartNextPacket',
    82:'IoStartNextPacketByKey', 83:'IoStartPacket', 84:'IoSynchronousDeviceIoControlRequest',
    85:'IoSynchronousFsdRequest', 86:'IofCallDriver', 87:'IofCompleteRequest',
    88:'KdDebuggerEnabled', 89:'KdDebuggerNotPresent', 90:'IoDismountVolume',
    91:'IoDismountVolumeByName', 92:'KeAlertResumeThread', 93:'KeAlertThread',
    94:'KeBoostPriorityThread', 95:'KeBugCheck', 96:'KeBugCheckEx', 97:'KeCancelTimer',
    98:'KeConnectInterrupt', 99:'KeDelayExecutionThread', 100:'KeDisconnectInterrupt',
    101:'KeEnterCriticalRegion', 102:'MmGlobalData', 103:'KeGetCurrentIrql',
    104:'KeGetCurrentThread', 105:'KeInitializeApc', 106:'KeInitializeDeviceQueue',
    107:'KeInitializeDpc', 108:'KeInitializeEvent', 109:'KeInitializeInterrupt',
    110:'KeInitializeMutant', 111:'KeInitializeQueue', 112:'KeInitializeSemaphore',
    113:'KeInitializeTimerEx', 114:'KeInsertByKeyDeviceQueue', 115:'KeInsertDeviceQueue',
    116:'KeInsertHeadQueue', 117:'KeInsertQueue', 118:'KeInsertQueueApc', 119:'KeInsertQueueDpc',
    120:'KeInterruptTime', 121:'KeIsExecutingDpc', 122:'KeLeaveCriticalRegion', 123:'KePulseEvent',
    124:'KeQueryBasePriorityThread', 125:'KeQueryInterruptTime', 126:'KeQueryPerformanceCounter',
    127:'KeQueryPerformanceFrequency', 128:'KeQuerySystemTime', 129:'KeRaiseIrqlToDpcLevel',
    130:'KeRaiseIrqlToSynchLevel', 131:'KeReleaseMutant', 132:'KeReleaseSemaphore',
    133:'KeRemoveByKeyDeviceQueue', 134:'KeRemoveDeviceQueue', 135:'KeRemoveEntryDeviceQueue',
    136:'KeRemoveQueue', 137:'KeRemoveQueueDpc', 138:'KeResetEvent',
    139:'KeRestoreFloatingPointState', 140:'KeResumeThread', 141:'KeRundownQueue',
    142:'KeSaveFloatingPointState', 143:'KeSetBasePriorityThread', 144:'KeSetDisableBoostThread',
    145:'KeSetEvent', 146:'KeSetEventBoostPriority', 147:'KeSetPriorityProcess',
    148:'KeSetPriorityThread', 149:'KeSetTimer', 150:'KeSetTimerEx',
    151:'KeStallExecutionProcessor', 152:'KeSuspendThread', 153:'KeSynchronizeExecution',
    154:'KeSystemTime', 155:'KeTestAlertThread', 156:'KeTickCount', 157:'KeTimeIncrement',
    158:'KeWaitForMultipleObjects', 159:'KeWaitForSingleObject', 160:'KfRaiseIrql',
    161:'KfLowerIrql', 162:'KiBugCheckData', 163:'KiUnlockDispatcherDatabase',
    164:'LaunchDataPage', 165:'MmAllocateContiguousMemory', 166:'MmAllocateContiguousMemoryEx',
    167:'MmAllocateSystemMemory', 168:'MmClaimGpuInstanceMemory', 169:'MmCreateKernelStack',
    170:'MmDeleteKernelStack', 171:'MmFreeContiguousMemory', 172:'MmFreeSystemMemory',
    173:'MmGetPhysicalAddress', 174:'MmIsAddressValid', 175:'MmLockUnlockBufferPages',
    176:'MmLockUnlockPhysicalPage', 177:'MmMapIoSpace', 178:'MmPersistContiguousMemory',
    179:'MmQueryAddressProtect', 180:'MmQueryAllocationSize', 181:'MmQueryStatistics',
    182:'MmSetAddressProtect', 183:'MmUnmapIoSpace', 184:'NtAllocateVirtualMemory',
    185:'NtCancelTimer', 186:'NtClearEvent', 187:'NtClose', 188:'NtCreateDirectoryObject',
    189:'NtCreateEvent', 190:'NtCreateFile', 191:'NtCreateIoCompletion', 192:'NtCreateMutant',
    193:'NtCreateSemaphore', 194:'NtCreateTimer', 195:'NtDeleteFile', 196:'NtDeviceIoControlFile',
    197:'NtDuplicateObject', 198:'NtFlushBuffersFile', 199:'NtFreeVirtualMemory',
    200:'NtFsControlFile', 201:'NtOpenDirectoryObject', 202:'NtOpenFile',
    203:'NtOpenSymbolicLinkObject', 204:'NtProtectVirtualMemory', 205:'NtPulseEvent',
    206:'NtQueueApcThread', 207:'NtQueryDirectoryFile', 208:'NtQueryDirectoryObject',
    209:'NtQueryEvent', 210:'NtQueryFullAttributesFile', 211:'NtQueryInformationFile',
    212:'NtQueryIoCompletion', 213:'NtQueryMutant', 214:'NtQuerySemaphore',
    215:'NtQuerySymbolicLinkObject', 216:'NtQueryTimer', 217:'NtQueryVirtualMemory',
    218:'NtQueryVolumeInformationFile', 219:'NtReadFile', 220:'NtReadFileScatter',
    221:'NtReleaseMutant', 222:'NtReleaseSemaphore', 223:'NtRemoveIoCompletion',
    224:'NtResumeThread', 225:'NtSetEvent', 226:'NtSetInformationFile', 227:'NtSetIoCompletion',
    228:'NtSetSystemTime', 229:'NtSetTimerEx', 230:'NtSignalAndWaitForSingleObjectEx',
    231:'NtSuspendThread', 232:'NtUserIoApcDispatcher', 233:'NtWaitForSingleObject',
    234:'NtWaitForSingleObjectEx', 235:'NtWaitForMultipleObjectsEx', 236:'NtWriteFile',
    237:'NtWriteFileGather', 238:'NtYieldExecution', 239:'ObCreateObject',
    240:'ObDirectoryObjectType', 241:'ObInsertObject', 242:'ObMakeTemporaryObject',
    243:'ObOpenObjectByName', 244:'ObOpenObjectByPointer', 245:'ObpObjectHandleTable',
    246:'ObReferenceObjectByHandle', 247:'ObReferenceObjectByName',
    248:'ObReferenceObjectByPointer', 249:'ObSymbolicLinkObjectType', 250:'ObfDereferenceObject',
    251:'ObfReferenceObject', 252:'PhyGetLinkState', 253:'PhyInitialize',
    254:'PsCreateSystemThread', 255:'PsCreateSystemThreadEx', 256:'PsQueryStatistics',
    257:'PsSetCreateThreadNotifyRoutine', 258:'PsTerminateSystemThread', 259:'PsThreadObjectType',
    260:'RtlAnsiStringToUnicodeString', 261:'RtlAppendStringToString',
    262:'RtlAppendUnicodeStringToString', 263:'RtlAppendUnicodeToString', 264:'RtlAssert',
    265:'RtlCaptureContext', 266:'RtlCaptureStackBackTrace', 267:'RtlCharToInteger',
    268:'RtlCompareMemory', 269:'RtlCompareMemoryUlong', 270:'RtlCompareString',
    271:'RtlCompareUnicodeString', 272:'RtlCopyString', 273:'RtlCopyUnicodeString',
    274:'RtlCreateUnicodeString', 275:'RtlDowncaseUnicodeChar', 276:'RtlDowncaseUnicodeString',
    277:'RtlEnterCriticalSection', 278:'RtlEnterCriticalSectionAndRegion', 279:'RtlEqualString',
    280:'RtlEqualUnicodeString', 281:'RtlExtendedIntegerMultiply',
    282:'RtlExtendedLargeIntegerDivide', 283:'RtlExtendedMagicDivide', 284:'RtlFillMemory',
    285:'RtlFillMemoryUlong', 286:'RtlFreeAnsiString', 287:'RtlFreeUnicodeString',
    288:'RtlGetCallersAddress', 289:'RtlInitAnsiString', 290:'RtlInitUnicodeString',
    291:'RtlInitializeCriticalSection', 292:'RtlIntegerToChar', 293:'RtlIntegerToUnicodeString',
    294:'RtlLeaveCriticalSection', 295:'RtlLeaveCriticalSectionAndRegion', 296:'RtlLowerChar',
    297:'RtlMapGenericMask', 298:'RtlMoveMemory', 299:'RtlMultiByteToUnicodeN',
    300:'RtlMultiByteToUnicodeSize', 301:'RtlNtStatusToDosError', 302:'RtlRaiseException',
    303:'RtlRaiseStatus', 304:'RtlTimeFieldsToTime', 305:'RtlTimeToTimeFields',
    306:'RtlTryEnterCriticalSection', 307:'RtlUlongByteSwap', 308:'RtlUnicodeStringToAnsiString',
    309:'RtlUnicodeStringToInteger', 310:'RtlUnicodeToMultiByteN', 311:'RtlUnicodeToMultiByteSize',
    312:'RtlUnwind', 313:'RtlUpcaseUnicodeChar', 314:'RtlUpcaseUnicodeString',
    315:'RtlUpcaseUnicodeToMultiByteN', 316:'RtlUpperChar', 317:'RtlUpperString',
    318:'RtlUshortByteSwap', 319:'RtlWalkFrameChain', 320:'RtlZeroMemory', 321:'XboxEEPROMKey',
    322:'XboxHardwareInfo', 323:'XboxHDKey', 324:'XboxKrnlVersion', 325:'XboxSignatureKey',
    326:'XeImageFileName', 327:'XeLoadSection', 328:'XeUnloadSection',
    329:'READ_PORT_BUFFER_UCHAR', 330:'READ_PORT_BUFFER_USHORT', 331:'READ_PORT_BUFFER_ULONG',
    332:'WRITE_PORT_BUFFER_UCHAR', 333:'WRITE_PORT_BUFFER_USHORT', 334:'WRITE_PORT_BUFFER_ULONG',
    335:'XcSHAInit', 336:'XcSHAUpdate', 337:'XcSHAFinal', 338:'XcRC4Key', 339:'XcRC4Crypt',
    340:'XcHMAC', 341:'XcPKEncPublic', 342:'XcPKDecPrivate', 343:'XcPKGetKeyLen',
    344:'XcVerifyPKCS1Signature', 345:'XcModExp', 346:'XcDESKeyParity', 347:'XcKeyTable',
    348:'XcBlockCrypt', 349:'XcBlockCryptCBC', 350:'XcCryptService', 351:'XcUpdateCrypto',
    352:'RtlRip', 353:'XboxLANKey', 354:'XboxAlternateSignatureKeys', 355:'XePublicKeyData',
    356:'HalBootSMCVideoMode', 357:'IdexChannelObject', 358:'HalIsResetOrShutdownPending',
    359:'IoMarkIrpMustComplete', 360:'HalInitiateShutdown', 361:'RtlSnprintf', 362:'RtlSprintf',
    363:'RtlVsnprintf', 364:'RtlVsprintf', 365:'HalEnableSecureTrayEject',
    366:'HalWriteSMCScratchRegister', 374:'MmDbgAllocateMemory', 375:'MmDbgFreeMemory',
    376:'MmDbgQueryAvailablePages', 377:'MmDbgReleaseAddress', 378:'MmDbgWriteCheck',
}


def kname(ordinal):
    return KERNEL_EXPORTS.get(ordinal, f'ord_{ordinal}')


def kgroup(name):
    for prefix, _ in sorted(KRNL_GROUPS, key=lambda g: -len(g[0])):
        if name.startswith(prefix):
            return prefix
    return 'other'


class XBE:
    def __init__(self, path):
        self.path = path
        self.data = open(path, 'rb').read()
        d = self.data
        if d[:4] != b'XBEH':
            raise ValueError(f'{path}: not an XBE (magic {d[:4]!r})')
        (self.base, self.size_headers, self.size_image, self.size_image_header, self.timedate,
         self.cert_addr, self.num_sections, self.section_headers_addr, self.init_flags,
         ep_raw, self.tls_addr, self.stack_commit, self.heap_reserve, self.heap_commit,
         self.pe_base, self.pe_size, self.pe_checksum, self.pe_timedate,
         self.debug_path_addr, self.debug_file_addr, self.debug_ufile_addr,
         kt_raw, self.nonkernel_import_addr, self.num_libs, self.libs_addr,
         self.kernel_lib_addr, self.xapi_lib_addr, self.logo_addr, self.logo_size
         ) = struct.unpack_from('<29I', d, 0x104)

        self.sections = []
        for i in range(self.num_sections):
            off = self.va2off(self.section_headers_addr) + i * 56
            flags, va, vsize, raw, rsize, name_addr, refcnt, head_ref, tail_ref = struct.unpack_from('<9I', d, off)
            self.sections.append(dict(index=i, flags=flags, va=va, vsize=vsize, raw=raw, rsize=rsize,
                                      name=self.cstr(name_addr), digest=d[off + 36:off + 56].hex()))

        # Decide retail vs debug by which XOR key gives a sane entry point.
        self.build = None
        for kind, ek, kk in (('retail', XOR_EP_RETAIL, XOR_KT_RETAIL), ('debug', XOR_EP_DEBUG, XOR_KT_DEBUG)):
            ep, kt = ep_raw ^ ek, kt_raw ^ kk
            if self.find_section(ep) and self.find_section(kt):
                self.build, self.entry, self.kthunk_addr = kind, ep, kt
                break
        if self.build is None:
            raise ValueError('could not decode entry point / kernel thunk with retail or debug keys')

        self.libs = []
        for i in range(self.num_libs):
            off = self.va2off(self.libs_addr) + i * 16
            name = d[off:off + 8].rstrip(b'\0').decode('ascii', 'replace')
            major, minor, build, flags = struct.unpack_from('<4H', d, off + 8)
            self.libs.append(dict(name=name, version=f'{major}.{minor}.{build}', build=build,
                                  qfe=flags & 0x1FFF, approved=(flags >> 13) & 3, debug=bool(flags >> 15)))

        self.kimports = []
        off = self.va2off(self.kthunk_addr)
        while True:
            (v,) = struct.unpack_from('<I', d, off)
            if v == 0:
                break
            ordinal = v & 0x7FFFFFFF if v & 0x80000000 else v
            self.kimports.append((ordinal, kname(ordinal)))
            off += 4

        self.cert = self.parse_cert()

    # ---- address helpers -------------------------------------------------
    def find_section(self, va):
        for s in self.sections:
            if s['va'] <= va < s['va'] + max(s['vsize'], s['rsize']):
                return s
        return None

    def va2off(self, va):
        if self.base <= va < self.base + self.size_headers:
            return va - self.base
        s = self.find_section(va)
        if s is None or va - s['va'] >= s['rsize']:
            raise ValueError(f'VA {va:#x} not backed by file data')
        return s['raw'] + (va - s['va'])

    def cstr(self, va, limit=256):
        try:
            off = self.va2off(va)
        except ValueError:
            return ''
        end = self.data.find(b'\0', off, off + limit)
        return self.data[off:end if end >= 0 else off + limit].decode('ascii', 'replace')

    def wstr(self, va, limit=512):
        try:
            off = self.va2off(va)
        except ValueError:
            return ''
        raw = self.data[off:off + limit]
        end = next((i for i in range(0, len(raw) - 1, 2) if raw[i:i + 2] == b'\0\0'), len(raw))
        return raw[:end].decode('utf-16-le', 'replace')

    def section_bytes(self, s):
        return self.data[s['raw']:s['raw'] + s['rsize']]

    def parse_cert(self):
        off = self.va2off(self.cert_addr)
        d = self.data
        size, timedate, title_id = struct.unpack_from('<3I', d, off)
        title = d[off + 0x0C:off + 0x0C + 80].decode('utf-16-le', 'replace').split('\0')[0]
        media, region, ratings, disk, version = struct.unpack_from('<5I', d, off + 0x9C)
        tid = struct.pack('>I', title_id)
        pub = tid[:2].decode('ascii', 'replace')
        return dict(size=size, title_id=f'{title_id:08X}', title_id_str=f'{pub}-{title_id & 0xFFFF:03d}',
                    title=title, timedate=timedate, media=flag_names(media, MEDIA),
                    region=flag_names(region, REGIONS), ratings=ratings, disk=disk, version=version)


def flag_names(value, table):
    names = [n for bit, n in table.items() if value & bit]
    rest = value & ~sum(table)
    if rest:
        names.append(f'{rest:#x}')
    return names


def fmt_flags(f):
    return ','.join(n for b, n in SECTION_FLAGS.items() if f & b) or '-'


def ts(t):
    import datetime
    try:
        return datetime.datetime.fromtimestamp(t, datetime.timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')
    except (OverflowError, OSError, ValueError):
        return f'{t:#x}'


def lib_note(name):
    for prefix in sorted(LIB_NOTES, key=len, reverse=True):
        if name.upper().startswith(prefix):
            return LIB_NOTES[prefix]
    return ''


# ---- commands --------------------------------------------------------------
def cmd_info(x, as_json):
    if as_json:
        out = {k: getattr(x, k) for k in ('path', 'build', 'base', 'entry', 'kthunk_addr', 'timedate',
                                          'stack_commit', 'heap_reserve', 'heap_commit', 'init_flags')}
        out.update(cert=x.cert, sections=x.sections, libs=x.libs,
                   kernel_imports=[dict(ordinal=o, name=n) for o, n in x.kimports],
                   debug_path=x.cstr(x.debug_path_addr))
        print(json.dumps(out, indent=2))
        return

    c = x.cert
    print(f'== {x.path}')
    print(f'Title        : {c["title"]}  [{c["title_id"]} / {c["title_id_str"]}]  version {c["version"]:#x}  disk {c["disk"]}')
    print(f'Region/media : {",".join(c["region"])}  /  {",".join(c["media"])}')
    print(f'Built        : {ts(x.timedate)}   ({x.build} XOR keys)')
    print(f'Debug path   : {x.cstr(x.debug_path_addr)}')
    print(f'Base / entry : {x.base:#010x} / {x.entry:#010x}   image size {x.size_image:#x}')
    print(f'Stack/heap   : stack commit {x.stack_commit:#x}, heap reserve {x.heap_reserve:#x}, commit {x.heap_commit:#x}')
    print(f'Init flags   : {x.init_flags:#x}   TLS {x.tls_addr:#x}   non-kernel imports {x.nonkernel_import_addr:#x}')

    print(f'\n-- Sections ({len(x.sections)})')
    print(f'  {"name":10} {"vaddr":>10} {"vsize":>9} {"raw":>9} {"rsize":>9}  flags')
    total = 0
    for s in x.sections:
        total += s['vsize']
        print(f'  {s["name"]:10} {s["va"]:#010x} {s["vsize"]:9,} {s["raw"]:#9x} {s["rsize"]:9,}  {fmt_flags(s["flags"])}')
    print(f'  total virtual size {total:,} bytes')

    print(f'\n-- XDK libraries ({len(x.libs)})')
    for l in x.libs:
        extra = ' DEBUG' if l['debug'] else ''
        approved = ['unapproved', 'possibly approved', 'approved', '?'][l['approved']]
        print(f'  {l["name"]:9} {l["version"]:12} qfe {l["qfe"]:<3} {approved:17}{extra}  {lib_note(l["name"])}')

    print(f'\n-- Kernel imports ({len(x.kimports)})  thunk table @ {x.kthunk_addr:#010x}')
    groups = defaultdict(list)
    for o, n in x.kimports:
        groups[kgroup(n)].append((o, n))
    notes = dict(KRNL_GROUPS)
    for g in sorted(groups, key=lambda g: -len(groups[g])):
        print(f'  [{g}] {len(groups[g])}  - {notes.get(g, "")}')
        names = [f'{n}({o})' for o, n in sorted(groups[g], key=lambda t: t[1])]
        line = '     '
        for nm in names:
            if len(line) + len(nm) > 110:
                print(line)
                line = '     '
            line += nm + ' '
        print(line)


# Hardware MMIO windows on the Xbox. NV2A sub-blocks are offsets from 0xFD000000.
HW_RANGES = [
    ('NV2A', 0xFD000000, 0xFE000000),
    ('APU', 0xFE800000, 0xFE880000),
    ('AC97', 0xFEC00000, 0xFEC01000),
    ('USB', 0xFED00000, 0xFED09000),
]
NV2A_BLOCKS = [
    (0x000000, 'PMC'), (0x001000, 'PBUS'), (0x002000, 'PFIFO'), (0x009000, 'PTIMER'),
    (0x100000, 'PFB'), (0x101000, 'PEXTDEV'), (0x400000, 'PGRAPH'), (0x600000, 'PCRTC'),
    (0x680000, 'PRAMDAC'), (0x700000, 'PRAMIN'), (0x800000, 'USER/DMA-put-get'),
]
HEX_RE = re.compile(r'0x([0-9a-f]{8})')
# Library sections in a statically-linked XBE: hits here are the XDK driver, replaced wholesale by HLE.
XDK_SECTIONS = ('D3D', 'D3DX', 'DSOUND', 'XGRPH', 'XPP', 'XONLINE', 'XNET', 'XACTENG', 'DOLBY', 'XMV', 'BINK')


def hw_label(v):
    for name, lo, hi in HW_RANGES:
        if lo <= v < hi:
            if name == 'NV2A':
                blk = [b for o, b in NV2A_BLOCKS if v - lo >= o][-1]
                return f'NV2A.{blk}'
            return name
    return None


def cmd_hwscan(x, verbose, verbose_all=False):
    """Disassemble executable sections (linear sweep) and report instructions whose
    immediate or memory operand is a hardware MMIO address.

    Brute Force links D3D8LTCG (link-time code generation): much of D3D is inlined into
    .text, so NV2A hits in .text are expected and do NOT necessarily mean game code pokes
    the GPU - but they do mean signature-based HLE (Cxbx-style) will miss those call sites."""
    try:
        import capstone
    except ImportError:
        sys.exit('hwscan needs capstone: pip install capstone')
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.skipdata = True
    print(f'== {x.path}  (capstone linear sweep; mem = [MMIO] access, base = round MMIO base loaded'
          f'{", imm = other mov/push immediates (often hashes)" if verbose_all else ""})')
    for s in x.sections:
        if not s['flags'] & 0x4 or s['name'] in ('.rdata', '.data') or s['name'].startswith('$$'):
            continue
        hits = Counter()
        sites = defaultdict(list)
        for ins in md.disasm_lite(x.section_bytes(s), s['va']):
            addr, size, mnem, ops = ins
            if mnem.startswith('.'):
                continue
            for m in HEX_RE.finditer(ops):
                v = int(m.group(1), 16)
                lab = hw_label(v)
                if not lab:
                    continue
                # Memory operand ([0xfd......]) = a real MMIO access. A bare immediate is only
                # "maybe": round base addresses count, arbitrary values are usually hashes/masks.
                inside_brackets = ops.rfind('[', 0, m.start()) > ops.rfind(']', 0, m.start())
                if not inside_brackets and v & 0xFFF and mnem not in ('mov', 'lea', 'push'):
                    continue
                kind = 'mem' if inside_brackets else ('imm' if v & 0xFFF else 'base')
                if kind == 'imm' and not verbose_all:
                    continue
                key = f'{lab}/{kind}'
                hits[key] += 1
                sites[key].append(f'{addr:#010x}: {mnem} {ops}')
        if not hits:
            continue
        lib = any(s['name'].startswith(p) for p in XDK_SECTIONS)
        tag = 'XDK lib' if lib else 'game/LTCG'
        print(f'  {s["name"]:9} [{tag:9}] ' + ', '.join(f'{k}: {v}' for k, v in sorted(hits.items())))
        if verbose:
            for lab in sorted(sites):
                for line in sites[lab][:verbose]:
                    print(f'      {lab:22} {line}')


def iter_strings(x, minlen, section=None):
    pat = re.compile(rb'[\x20-\x7e]{%d,}' % minlen)
    for s in x.sections:
        if section and s['name'] != section:
            continue
        b = x.section_bytes(s)
        for m in pat.finditer(b):
            yield s['name'], s['va'] + m.start(), m.group().decode('ascii')


def cmd_strings(x, minlen, section):
    for sec, va, txt in iter_strings(x, minlen, section):
        print(f'{va:#010x} {sec:8} {txt}')


def cmd_dump(x, section, out):
    for s in x.sections:
        if s['name'] == section:
            open(out, 'wb').write(x.section_bytes(s))
            print(f'wrote {s["rsize"]:,} bytes of {section} (VA {s["va"]:#x}) to {out}')
            return
    sys.exit(f'no section named {section!r}; have: {", ".join(s["name"] for s in x.sections)}')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('info'); p.add_argument('xbe'); p.add_argument('--json', action='store_true')
    p = sub.add_parser('hwscan'); p.add_argument('xbe')
    p.add_argument('-v', type=int, default=0, metavar='N', help='show first N instruction sites per block')
    p.add_argument('--all', action='store_true', help='also count non-round mov/push immediates (noisy)')
    p = sub.add_parser('strings'); p.add_argument('xbe'); p.add_argument('-n', type=int, default=6)
    p.add_argument('--section')
    p = sub.add_parser('dump'); p.add_argument('xbe'); p.add_argument('section'); p.add_argument('out')
    a = ap.parse_args()

    x = XBE(a.xbe)
    if a.cmd == 'info':
        cmd_info(x, a.json)
    elif a.cmd == 'hwscan':
        cmd_hwscan(x, a.v, a.all)
    elif a.cmd == 'strings':
        cmd_strings(x, a.n, a.section)
    elif a.cmd == 'dump':
        cmd_dump(x, a.section, a.out)


if __name__ == '__main__':
    main()
