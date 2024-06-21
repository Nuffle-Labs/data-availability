package plasma_test

import (
	"testing"

	"github.com/ethereum-optimism/optimism/op-e2e/actions"
	"github.com/ethereum-optimism/optimism/op-e2e/config"
	"github.com/ethereum-optimism/optimism/op-e2e/e2eutils"
	plasma "github.com/ethereum-optimism/optimism/op-plasma"
	"github.com/ethereum-optimism/optimism/op-service/sources"
	"github.com/ethereum-optimism/optimism/op-service/testlog"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/log"
	"github.com/stretchr/testify/require"
)

var defaultAlloc = &e2eutils.AllocParams{PrefundTestUsers: true}

// L2PlasmaDA is a test harness for manipulating plasma DA state.
type L2PlasmaDA struct {
	log       log.Logger
	storage   *plasma.DAClient
	daMgr     *plasma.DA
	plasmaCfg plasma.Config
	batcher   *actions.L2Batcher
	sequencer *actions.L2Sequencer
	engine    *actions.L2Engine
	engCl     *sources.EngineClient
	sd        *e2eutils.SetupData
	dp        *e2eutils.DeployParams
	miner     *actions.L1Miner
}

func (a *L2PlasmaDA) ActL1Blocks(t actions.Testing, n uint64) {
	for i := uint64(0); i < n; i++ {
		a.miner.ActL1StartBlock(12)(t)
		a.miner.ActL1EndBlock(t)
	}
}

func NewL2PlasmaDA(t actions.Testing, daHost string) *L2PlasmaDA {
	p := &e2eutils.TestParams{
		MaxSequencerDrift:   40,
		SequencerWindowSize: 120,
		ChannelTimeout:      120,
		L1BlockTime:         12,
		UsePlasma:           true,
	}

	log := testlog.Logger(t, log.LvlDebug)

	config.DeployConfig.DACommitmentType = plasma.GenericCommitmentString
	dp := e2eutils.MakeDeployParams(t, p)
	dp.DeployConfig.DAChallengeProxy = common.Address{0x42}
	sd := e2eutils.Setup(t, dp, defaultAlloc)

	require.True(t, sd.RollupCfg.PlasmaEnabled())

	miner := actions.NewL1Miner(t, log, sd.L1Cfg)
	l1Client := miner.EthClient()

	jwtPath := e2eutils.WriteDefaultJWT(t)
	engine := actions.NewL2Engine(t, log, sd.L2Cfg, sd.RollupCfg.Genesis.L1, jwtPath)
	engCl := engine.EngineClient(t, sd.RollupCfg)

	storage := plasma.NewDAClient(daHost, false, false)

	l1F, err := sources.NewL1Client(miner.RPCClient(), log, nil, sources.L1ClientDefaultConfig(sd.RollupCfg, false, sources.RPCKindBasic))
	require.NoError(t, err)

	plasmaCfg, err := sd.RollupCfg.GetOPPlasmaConfig()
	require.NoError(t, err)

	plasmaCfg.CommitmentType = plasma.GenericCommitmentType

	daMgr := plasma.NewPlasmaDAWithStorage(log, plasmaCfg, storage, &plasma.NoopMetrics{})

	enabled := sd.RollupCfg.PlasmaEnabled()
	require.True(t, enabled)

	sequencer := actions.NewL2Sequencer(t, log, l1F, nil, daMgr, engCl, sd.RollupCfg, 0)
	miner.ActL1SetFeeRecipient(common.Address{'A'})
	sequencer.ActL2PipelineFull(t)

	batcher := actions.NewL2Batcher(log, sd.RollupCfg, actions.PlasmaBatcherCfg(dp, storage), sequencer.RollupClient(), l1Client, engine.EthClient(), engCl)

	return &L2PlasmaDA{
		log:       log,
		storage:   storage,
		daMgr:     daMgr,
		plasmaCfg: plasmaCfg,
		batcher:   batcher,
		sequencer: sequencer,
		engine:    engine,
		engCl:     engCl,
		sd:        sd,
		dp:        dp,
		miner:     miner,
	}
}

func (a *L2PlasmaDA) ActL1Finalized(t actions.Testing) {
	latest := uint64(2)
	a.miner.ActL1Safe(t, latest)
	a.miner.ActL1Finalize(t, latest)
	a.sequencer.ActL1FinalizedSignal(t)
}

func TestOptimism(gt *testing.T) {
	if !isE2e && !runTestnetIntegrationTests {
		gt.Skip("Skipping test as INTEGRATION or TESTNET env var not set")
	}

	proxyTS, close := e2e.CreateTestSuite(gt, true)
	defer close()

	t := actions.NewDefaultTesting(gt)

	op_stack := NewL2PlasmaDA(t, proxyTS.Address())

	// build L1 block #1
	op_stack.ActL1Blocks(t, 1)
	op_stack.miner.ActL1SafeNext(t)

	// Fill with l2 blocks up to the L1 head
	op_stack.sequencer.ActL1HeadSignal(t)
	op_stack.sequencer.ActBuildToL1Head(t)

	op_stack.sequencer.ActL2PipelineFull(t)
	op_stack.sequencer.ActL1SafeSignal(t)
	require.Equal(t, uint64(1), op_stack.sequencer.SyncStatus().SafeL1.Number)

	// add L1 block #2
	op_stack.ActL1Blocks(t, 1)
	op_stack.miner.ActL1SafeNext(t)
	op_stack.miner.ActL1FinalizeNext(t)
	op_stack.sequencer.ActL1HeadSignal(t)
	op_stack.sequencer.ActBuildToL1Head(t)

	// Catch up derivation
	op_stack.sequencer.ActL2PipelineFull(t)
	op_stack.sequencer.ActL1FinalizedSignal(t)
	op_stack.sequencer.ActL1SafeSignal(t)

	// commit all the l2 blocks to L1
	op_stack.batcher.ActSubmitAll(t)
	op_stack.miner.ActL1StartBlock(12)(t)
	op_stack.miner.ActL1IncludeTx(op_stack.dp.Addresses.Batcher)(t)
	op_stack.miner.ActL1EndBlock(t)

	// verify
	op_stack.sequencer.ActL2PipelineFull(t)
	op_stack.ActL1Finalized(t)

	// assert that EigenDA proxy's was written and read from
	stat := proxyTS.Server.Store().Stats()

	require.Equal(t, 1, stat.Entries)
	require.Equal(t, 1, stat.Reads)
}
